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

use crate::{lowerer::ExprLowerer, lowering::lower_type, MResult};

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
        if let ItemKind::Struct(struct_def) = &item.kind {
            if let VariantData::Struct { fields } = &struct_def.data {
                let name = struct_def.ident.name.text(db).to_string();
                let field_map = fields
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, f)| f.ident.as_ref().map(|id| (id.name, idx)))
                    .collect();
                struct_field_maps.insert(name, field_map);
            }
        }
    }

    for item in ast_items {
        match &item.kind {
            ItemKind::Fn(fn_def) => {
                let mir_function =
                    lower_function(db, *fn_def, source, type_table, &struct_field_maps)?;
                items.push(ir::Items::Function(mir_function));
            }
            ItemKind::ForeignMod(foreign_mod) => {
                for foreign_item in foreign_mod.items.iter() {
                    let sig = lower_foreign_signature(db, foreign_item)?;
                    let extern_fn = ir::ExternFn::new(db, foreign_mod.abi, sig);
                    items.push(ir::Items::ExternFunction(extern_fn));
                }
            }
            ItemKind::Struct(struct_def) => {
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
            _ => {
                continue;
            }
        }
    }

    Ok(ir::Module::new(db, module_id, items))
}

/// Lower a function definition
pub fn lower_function<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
    struct_field_maps: &HashMap<String, HashMap<Symbol<'db>, usize>>,
) -> MResult<ir::Function<'db>> {
    let signature = lower_signature(db, ast_function, type_table)?;
    let return_ty = signature.return_ty(db);
    let body = lower_body(db, ast_function, source, type_table, return_ty, struct_field_maps)?;

    Ok(ir::Function::new(db, signature, body))
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
        let param_ty = lower_type(db, &*arg.ty)?;
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
        let param_ty = lower_type(db, &*arg.ty)?;
        params.push(param_ty);
    }

    let return_ty = match item.ret_type.as_ref() {
        Some(ty) => lower_type(db, ty)?,
        None => ir::Ty::Void,
    };

    Ok(ir::Signature::new(db, name, params, return_ty))
}

/// Lower function body using ExprLowerer for proper expression handling
pub fn lower_body<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
    return_ty: ir::Ty<'db>,
    struct_field_maps: &HashMap<String, HashMap<Symbol<'db>, usize>>,
) -> MResult<ir::Body<'db>> {
    let mut lowerer = ExprLowerer::new(db, source, type_table);
    lowerer.struct_fields = struct_field_maps.clone();

    // _0 is always the return place
    let is_void_return = matches!(return_ty, ir::Ty::Void);
    lowerer.allocate_temp(return_ty);

    // _1, _2, ... are function parameters
    let param_count = ast_function.args(db).len();
    for param in ast_function.args(db).iter() {
        let param_ty = lower_type(db, &*param.ty)?;
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

    // 4. Build the CFG and return the body
    let blocks = lowerer.cfg_builder.build();
    Ok(ir::Body::new(db, blocks, lowerer.local_decls, param_count))
}
