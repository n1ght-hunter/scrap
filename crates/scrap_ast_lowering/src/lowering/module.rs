//! Module and function lowering from AST to IR

use std::collections::HashMap;

use scrap_ast::{
    enumdef::VariantData,
    fndef::FnDef,
    foreign::ForeignItem,
    item::{Item, ItemKind},
    local::LocalKind,
    pat::PatKind,
    stmt::StmtKind,
};
use scrap_ir as ir;
use scrap_shared::id::ModuleId;
use scrap_shared::ident::Symbol;

use crate::{MResult, lowerer::ExprLowerer, lowering::lower_type, lowering::lower_type_with_subst};

/// Lower a module with its items
pub fn lower_module<'db>(
    db: &'db dyn scrap_shared::Db,
    module_id: ModuleId<'db>,
    ast_items: &[Item<'db>],
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
) -> MResult<ir::Module<'db>> {
    let mut items = Vec::new();

    // Collect struct field maps for expression lowering (field name → index)
    let mut struct_field_maps: HashMap<String, HashMap<Symbol<'db>, usize>> = HashMap::new();
    for item in ast_items {
        if let ItemKind::Struct(struct_def) = &item.kind
            && let VariantData::Struct { fields } = &struct_def.data
        {
            let name = struct_def.ident.name.text(db).to_string();
            let field_map = fields
                .iter()
                .enumerate()
                .filter_map(|(idx, f)| f.ident.as_ref().map(|id| (id.name, idx)))
                .collect();
            struct_field_maps.insert(name, field_map);
        }
    }

    // Pre-register generic struct field maps for monomorphized copies
    for &(inst_name, _, ref subst_pairs) in type_table.generic_instantiations(db) {
        let inst_str = inst_name.text(db).to_string();
        if let Some(base_map) = struct_field_maps.get(&inst_str).cloned() {
            let mangled = mangle_generic_name(db, inst_name, subst_pairs);
            struct_field_maps.insert(mangled, base_map);
        }
    }

    // Collect enum variant maps for expression lowering
    let mut enum_info_maps: HashMap<String, crate::lowerer::EnumInfo<'db>> = HashMap::new();
    for item in ast_items {
        if let ItemKind::Enum(enum_def) = &item.kind {
            let name = enum_def.ident.name.text(db).to_string();
            let variants: Vec<_> = enum_def
                .variants
                .iter()
                .enumerate()
                .map(|(idx, variant)| {
                    let info = match &variant.data {
                        VariantData::Unit(_) => crate::lowerer::VariantInfo::Unit,
                        VariantData::Tuple(fields, _) => {
                            let tys = fields
                                .iter()
                                .map(|f| lower_type(db, &f.ty).unwrap_or(ir::Ty::Void))
                                .collect();
                            crate::lowerer::VariantInfo::Tuple(tys)
                        }
                        VariantData::Struct { fields } => {
                            let defs = fields
                                .iter()
                                .filter_map(|f| {
                                    let n = f.ident.as_ref()?.name;
                                    let t = lower_type(db, &f.ty).unwrap_or(ir::Ty::Void);
                                    Some((n, t))
                                })
                                .collect();
                            crate::lowerer::VariantInfo::Struct(defs)
                        }
                    };
                    (variant.ident.name, idx, info)
                })
                .collect();
            enum_info_maps.insert(name, crate::lowerer::EnumInfo { variants });
        }
    }

    let mut generic_fndefs: HashMap<Symbol<'db>, FnDef<'db>> = HashMap::new();
    let mut generic_structdefs: HashMap<String, &scrap_ast::structdef::StructDef<'db>> =
        HashMap::new();

    for item in ast_items {
        match &item.kind {
            ItemKind::Fn(fn_def) => {
                if !fn_def.generics(db).is_empty() {
                    generic_fndefs.insert(fn_def.ident(db).name, *fn_def);
                    continue;
                }
                let (mir_function, extras) = lower_function(
                    db,
                    *fn_def,
                    source,
                    type_table,
                    &struct_field_maps,
                    &enum_info_maps,
                )?;
                items.push(ir::Items::Function(mir_function));
                items.extend(extras);
            }
            ItemKind::ForeignMod(foreign_mod) => {
                for foreign_item in foreign_mod.items.iter() {
                    let sig = lower_foreign_signature(db, foreign_item)?;
                    let extern_fn = ir::ExternFn::new(db, foreign_mod.abi, sig);
                    items.push(ir::Items::ExternFunction(extern_fn));
                }
            }
            ItemKind::Struct(struct_def) => {
                if !struct_def.generics.is_empty() {
                    generic_structdefs
                        .insert(struct_def.ident.name.text(db).to_string(), struct_def);
                    continue;
                }
                if let VariantData::Struct { fields } = &struct_def.data {
                    let name = struct_def.ident.name;
                    let ir_fields: Vec<(Symbol<'db>, ir::Ty<'db>)> = fields
                        .iter()
                        .filter_map(|field| {
                            let field_name = field.ident.as_ref()?.name;
                            let field_ty = lower_type(db, &field.ty).unwrap_or(ir::Ty::Void);
                            Some((field_name, field_ty))
                        })
                        .collect();
                    let ir_struct = ir::Struct::new(db, name, ir_fields);
                    items.push(ir::Items::Struct(ir_struct));
                }
            }
            ItemKind::Enum(enum_def) => {
                if !enum_def.generics.is_empty() {
                    continue;
                }
                let name = enum_def.ident.name;
                let ir_variants: Vec<ir::EnumVariant<'db>> = enum_def
                    .variants
                    .iter()
                    .map(|variant| match &variant.data {
                        VariantData::Unit(_) => ir::EnumVariant::Unit(variant.ident.name),
                        VariantData::Tuple(fields, _) => {
                            let field_tys: Vec<ir::Ty<'db>> = fields
                                .iter()
                                .map(|f| lower_type(db, &f.ty).unwrap_or(ir::Ty::Void))
                                .collect();
                            ir::EnumVariant::Tuple(variant.ident.name, field_tys)
                        }
                        VariantData::Struct { fields } => {
                            let field_defs: Vec<(Symbol<'db>, ir::Ty<'db>)> = fields
                                .iter()
                                .filter_map(|f| {
                                    let n = f.ident.as_ref()?.name;
                                    let t = lower_type(db, &f.ty).unwrap_or(ir::Ty::Void);
                                    Some((n, t))
                                })
                                .collect();
                            ir::EnumVariant::Struct(variant.ident.name, field_defs)
                        }
                    })
                    .collect();
                let ir_enum = ir::Enum::new(db, name, ir_variants);
                items.push(ir::Items::Enum(ir_enum));
            }
            ItemKind::Impl(impl_block) => {
                for method in &impl_block.methods {
                    let (mir_fn, extras) = lower_method(
                        db,
                        impl_block.type_name.name,
                        *method,
                        source,
                        type_table,
                        &struct_field_maps,
                        &enum_info_maps,
                    )?;
                    items.push(ir::Items::Function(mir_fn));
                    items.extend(extras);
                }
            }
            _ => {
                continue;
            }
        }
    }

    // Monomorphize generics: generate concrete copies for each instantiation
    let mut seen_mono = std::collections::HashSet::new();
    for &(inst_name, _, ref subst_pairs) in type_table.generic_instantiations(db) {
        let mangled = mangle_generic_name(db, inst_name, subst_pairs);
        if !seen_mono.insert(mangled.clone()) {
            continue;
        }

        let type_subst: HashMap<Symbol<'db>, ir::Ty<'db>> = subst_pairs
            .iter()
            .map(|(param, resolved)| (*param, crate::ty_convert::resolved_to_ir(db, resolved)))
            .collect();

        // Generic function
        if let Some(&fn_def) = generic_fndefs.get(&inst_name) {
            let mangled_sym = Symbol::new(db, mangled.clone());
            let (mir_fn, extras) = lower_monomorphized_function(
                db,
                fn_def,
                mangled_sym,
                &type_subst,
                source,
                type_table,
                &struct_field_maps,
                &enum_info_maps,
            )?;
            items.push(ir::Items::Function(mir_fn));
            items.extend(extras);
        }

        // Generic struct
        let inst_name_str = inst_name.text(db).to_string();
        if let Some(struct_def) = generic_structdefs.get(&inst_name_str)
            && let VariantData::Struct { fields } = &struct_def.data
        {
            let mangled_sym = Symbol::new(db, mangled.clone());
            let ir_fields: Vec<(Symbol<'db>, ir::Ty<'db>)> = fields
                .iter()
                .filter_map(|field| {
                    let field_name = field.ident.as_ref()?.name;
                    let field_ty =
                        lower_type_with_subst(db, &field.ty, &type_subst).unwrap_or(ir::Ty::Void);
                    Some((field_name, field_ty))
                })
                .collect();
            let ir_struct = ir::Struct::new(db, mangled_sym, ir_fields);
            items.push(ir::Items::Struct(ir_struct));

            // Register in struct_field_maps for field access lowering
            let field_map = fields
                .iter()
                .enumerate()
                .filter_map(|(idx, f)| f.ident.as_ref().map(|id| (id.name, idx)))
                .collect();
            struct_field_maps.insert(mangled, field_map);
        }
    }

    Ok(ir::Module::new(db, module_id, items))
}

pub(crate) fn mangle_generic_name_from_resolved<'db>(
    db: &'db dyn scrap_shared::Db,
    base_name: Symbol<'db>,
    args: &[scrap_tycheck::ResolvedTy<'db>],
) -> String {
    let mut name = base_name.text(db).to_string();
    for ty in args {
        name.push_str("__");
        name.push_str(&mangle_type(ty));
    }
    name
}

pub(crate) fn mangle_generic_name<'db>(
    db: &'db dyn scrap_shared::Db,
    fn_name: Symbol<'db>,
    subst: &[(Symbol<'db>, scrap_tycheck::ResolvedTy<'db>)],
) -> String {
    let mut name = fn_name.text(db).to_string();
    for (_, ty) in subst {
        name.push_str("__");
        name.push_str(&mangle_type(ty));
    }
    name
}

fn mangle_type(ty: &scrap_tycheck::ResolvedTy) -> String {
    use scrap_shared::types::*;
    match ty {
        scrap_tycheck::ResolvedTy::Void => "void".into(),
        scrap_tycheck::ResolvedTy::Bool => "bool".into(),
        scrap_tycheck::ResolvedTy::Int(k) => match k {
            IntTy::I8 => "i8",
            IntTy::I16 => "i16",
            IntTy::I32 => "i32",
            IntTy::I64 => "i64",
            IntTy::I128 => "i128",
            IntTy::Isize => "isize",
        }
        .into(),
        scrap_tycheck::ResolvedTy::Uint(k) => match k {
            UintTy::U8 => "u8",
            UintTy::U16 => "u16",
            UintTy::U32 => "u32",
            UintTy::U64 => "u64",
            UintTy::U128 => "u128",
            UintTy::Usize => "usize",
        }
        .into(),
        scrap_tycheck::ResolvedTy::Float(k) => match k {
            FloatTy::F16 => "f16",
            FloatTy::F32 => "f32",
            FloatTy::F64 => "f64",
            FloatTy::F128 => "f128",
        }
        .into(),
        scrap_tycheck::ResolvedTy::Str => "String".into(),
        scrap_tycheck::ResolvedTy::Never => "never".into(),
        scrap_tycheck::ResolvedTy::Ref(inner, m) => {
            let prefix = if *m == Mutability::Mut {
                "ref_mut_"
            } else {
                "ref_"
            };
            format!("{}{}", prefix, mangle_type(inner))
        }
        scrap_tycheck::ResolvedTy::Ptr(inner) => format!("ptr_{}", mangle_type(inner)),
        _ => format!("{:?}", ty),
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_monomorphized_function<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
    mangled_name: Symbol<'db>,
    type_subst: &HashMap<Symbol<'db>, ir::Ty<'db>>,
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
    struct_field_maps: &HashMap<String, HashMap<Symbol<'db>, usize>>,
    enum_info_maps: &HashMap<String, crate::lowerer::EnumInfo<'db>>,
) -> MResult<(ir::Function<'db>, Vec<ir::Items<'db>>)> {
    let mut params = Vec::new();
    for arg in ast_function.args(db).iter() {
        let param_ty = lower_type_with_subst(db, &arg.ty, type_subst)?;
        params.push(param_ty);
    }

    let return_ty = match ast_function.ret_type(db).as_ref() {
        Some(ty) => lower_type_with_subst(db, ty, type_subst)?,
        None => ir::Ty::Void,
    };

    let signature = ir::Signature::new(db, mangled_name, params, return_ty.clone());
    let (body, extras) = lower_body_with_subst(
        db,
        ast_function,
        source,
        type_table,
        return_ty,
        struct_field_maps,
        enum_info_maps,
        type_subst,
    )?;

    Ok((ir::Function::new(db, signature, body), extras))
}

/// Lower a function definition, returning the function and any extra functions
/// generated by `spawn { block }` expressions.
pub fn lower_function<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
    struct_field_maps: &HashMap<String, HashMap<Symbol<'db>, usize>>,
    enum_info_maps: &HashMap<String, crate::lowerer::EnumInfo<'db>>,
) -> MResult<(ir::Function<'db>, Vec<ir::Items<'db>>)> {
    let signature = lower_signature(db, ast_function, type_table)?;
    let return_ty = signature.return_ty(db);
    let (body, extras) = lower_body(
        db,
        ast_function,
        source,
        type_table,
        return_ty,
        struct_field_maps,
        enum_info_maps,
    )?;

    Ok((ir::Function::new(db, signature, body), extras))
}

/// Lower function signature
pub fn lower_signature<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
    type_table: scrap_tycheck::TypeTable<'db>,
) -> MResult<ir::Signature<'db>> {
    let name = ast_function.ident(db).name;

    let mut params = Vec::new();
    for arg in ast_function.args(db).iter() {
        let param_ty = lower_type(db, &arg.ty)?;
        params.push(param_ty);
    }

    let return_ty = match ast_function.ret_type(db).as_ref() {
        Some(ty) => lower_type(db, ty)?,
        None => {
            // No explicit return type — check if the type checker inferred one
            type_table
                .fn_return_type(db, name)
                .map(|resolved| crate::ty_convert::resolved_to_ir(db, resolved))
                .unwrap_or(ir::Ty::Void)
        }
    };

    Ok(ir::Signature::new(db, name, params, return_ty))
}

/// Lower a foreign (extern) function signature
pub fn lower_foreign_signature<'db>(
    db: &'db dyn scrap_shared::Db,
    item: &ForeignItem<'db>,
) -> MResult<ir::Signature<'db>> {
    let name = item.ident.name;

    let mut params = Vec::new();
    for arg in item.args.iter() {
        let param_ty = lower_type(db, &arg.ty)?;
        params.push(param_ty);
    }

    let return_ty = match item.ret_type.as_ref() {
        Some(ty) => lower_type(db, ty)?,
        None => ir::Ty::Void,
    };

    Ok(ir::Signature::new(db, name, params, return_ty))
}

/// Lower a method definition (same as a function but with a mangled name).
pub fn lower_method<'db>(
    db: &'db dyn scrap_shared::Db,
    type_name: scrap_shared::ident::Symbol<'db>,
    ast_function: FnDef<'db>,
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
    struct_field_maps: &HashMap<String, HashMap<Symbol<'db>, usize>>,
    enum_info_maps: &HashMap<String, crate::lowerer::EnumInfo<'db>>,
) -> MResult<(ir::Function<'db>, Vec<ir::Items<'db>>)> {
    let method_name = ast_function.ident(db).name;
    let mangled = Symbol::new(
        db,
        format!("{}::{}", type_name.text(db), method_name.text(db)),
    );
    let signature = lower_signature_with_name(db, mangled, ast_function, type_table)?;
    let return_ty = signature.return_ty(db);
    let (body, extras) = lower_body(
        db,
        ast_function,
        source,
        type_table,
        return_ty,
        struct_field_maps,
        enum_info_maps,
    )?;

    Ok((ir::Function::new(db, signature, body), extras))
}

/// Lower a function signature using an explicit name (for methods with mangled names).
pub fn lower_signature_with_name<'db>(
    db: &'db dyn scrap_shared::Db,
    name: Symbol<'db>,
    ast_function: FnDef<'db>,
    type_table: scrap_tycheck::TypeTable<'db>,
) -> MResult<ir::Signature<'db>> {
    let mut params = Vec::new();
    for arg in ast_function.args(db).iter() {
        let param_ty = lower_type(db, &arg.ty)?;
        params.push(param_ty);
    }

    let return_ty = match ast_function.ret_type(db).as_ref() {
        Some(ty) => lower_type(db, ty)?,
        None => type_table
            .fn_return_type(db, name)
            .map(|resolved| crate::ty_convert::resolved_to_ir(db, resolved))
            .unwrap_or(ir::Ty::Void),
    };

    Ok(ir::Signature::new(db, name, params, return_ty))
}

/// Lower function body using ExprLowerer for proper expression handling.
/// Returns the body and any extra functions generated by `spawn { block }`.
pub fn lower_body<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
    return_ty: ir::Ty<'db>,
    struct_field_maps: &HashMap<String, HashMap<Symbol<'db>, usize>>,
    enum_info_maps: &HashMap<String, crate::lowerer::EnumInfo<'db>>,
) -> MResult<(ir::Body<'db>, Vec<ir::Items<'db>>)> {
    lower_body_with_subst(
        db,
        ast_function,
        source,
        type_table,
        return_ty,
        struct_field_maps,
        enum_info_maps,
        &HashMap::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn lower_body_with_subst<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
    return_ty: ir::Ty<'db>,
    struct_field_maps: &HashMap<String, HashMap<Symbol<'db>, usize>>,
    enum_info_maps: &HashMap<String, crate::lowerer::EnumInfo<'db>>,
    type_subst: &HashMap<Symbol<'db>, ir::Ty<'db>>,
) -> MResult<(ir::Body<'db>, Vec<ir::Items<'db>>)> {
    let mut lowerer = ExprLowerer::new(db, source, type_table);
    lowerer.struct_fields = struct_field_maps.clone();
    lowerer.enum_info = enum_info_maps.clone();

    // _0 is always the return place
    let is_void_return = matches!(return_ty, ir::Ty::Void);
    lowerer.allocate_temp(return_ty);

    // _1, _2, ... are function parameters
    let param_count = ast_function.args(db).len();
    for param in ast_function.args(db).iter() {
        let param_ty = lower_type_with_subst(db, &param.ty, type_subst)?;
        let local_id = lowerer.allocate_named_local(param.ident.name, param_ty);
        lowerer.insert_binding(param.ident.name, local_id);
    }

    // 2. Process all statements in the body
    let body = ast_function.body(db);
    let stmts = &body.stmts;
    let last_idx = stmts.len().saturating_sub(1);
    for (idx, stmt) in stmts.iter().enumerate() {
        let is_last = idx == last_idx;
        match &stmt.kind {
            StmtKind::Let(local) => {
                // Handle let bindings
                if let PatKind::Ident(_, ident, _) = &local.pat.kind {
                    // Get type from explicit annotation or type table
                    let ty = if let Some(explicit_ty) = local.ty.as_ref() {
                        lower_type(db, explicit_ty)?
                    } else {
                        // No explicit type - look up from type table using local's NodeId
                        lowerer.lookup_and_convert_local_type(local.id)
                    };

                    let local_id = lowerer.allocate_named_local(ident.name, ty);
                    lowerer.insert_binding(ident.name, local_id);

                    // If there's an initializer, lower it directly into the local
                    if let LocalKind::Init(init) = &local.kind {
                        lowerer.lower_expr_into(init, ir::Place::Local(local_id))?;
                    }
                }
            }
            StmtKind::Expr(expr) if is_last && !is_void_return => {
                // Last expression without semicolon in a non-void function:
                // this is an implicit return — assign result directly to _0
                let ret_place = lowerer.return_place();
                lowerer.lower_expr_into(expr, ret_place)?;
            }
            StmtKind::Semi(expr) | StmtKind::Expr(expr) => {
                lowerer.lower_expr(expr)?;
            }
            StmtKind::Item(_) | StmtKind::Empty => {
                // Skip items and empty statements
            }
        }
    }

    // 3. Ensure the final block is terminated
    if !lowerer.cfg_builder.current_block_is_terminated() {
        lowerer.cfg_builder.finish_block(ir::Terminator::Return);
    }

    // 4. Build the CFG and return the body + any extra functions from spawn blocks
    let blocks = lowerer.cfg_builder.build();
    let extras = lowerer.extra_functions;
    Ok((
        ir::Body::new(db, blocks, lowerer.local_decls, param_count),
        extras,
    ))
}
