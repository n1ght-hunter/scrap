//! Top-level type checking for modules and items.

use scrap_ast::{
    Can,
    enumdef::VariantData,
    fndef::FnDef,
    foreign::ForeignItem,
    impl_block::ImplBlock,
    item::{Item, ItemKind},
    module::ModuleKind,
    structdef::StructDef,
};
use scrap_shared::ident::Symbol;

use crate::{
    context::{EnumVariantDef, FnSig, TypeContext},
    types::InferTy,
};

impl<'db> TypeContext<'db> {
    /// Type check an entire compilation unit.
    pub fn check_can(&mut self, can: Can<'db>) {
        let items = can.items(self.db());

        // First pass: collect all function signatures
        for item in items.iter() {
            self.collect_item_signature(item);
        }

        // Second pass: type check function bodies
        for item in items.iter() {
            self.check_item(item);
        }

        // Solve all collected constraints
        self.solve_constraints();
    }

    /// Collect the signature of an item (first pass).
    fn collect_item_signature(&mut self, item: &Item<'db>) {
        match &item.kind {
            ItemKind::Fn(fn_def) => {
                self.collect_fn_signature(*fn_def);
            }
            ItemKind::Struct(struct_def) => {
                self.collect_struct_definition(struct_def);
            }
            ItemKind::Enum(enum_def) => {
                self.collect_enum_definition(enum_def);
            }
            ItemKind::Module(module) => {
                // Recursively collect signatures from submodule
                if let ModuleKind::Loaded(items, _, _) = module.kind(self.db()) {
                    for sub_item in items.iter() {
                        self.collect_item_signature(sub_item);
                    }
                }
            }
            ItemKind::ForeignMod(foreign_mod) => {
                for item in foreign_mod.items.iter() {
                    self.collect_foreign_fn_signature(item);
                }
            }
            ItemKind::Use(_) => {
                // Use statements don't contribute type signatures
            }
            ItemKind::Impl(impl_block) => {
                self.collect_impl_signatures(impl_block);
            }
        }
    }

    /// Collect a foreign function signature.
    fn collect_foreign_fn_signature(&mut self, item: &ForeignItem<'db>) {
        let name = item.ident.name;

        let type_params = vec![];
        self.set_type_params(type_params.clone());

        let params: Vec<_> = item
            .args
            .iter()
            .map(|param| {
                let ty = self.lower_ast_ty(&param.ty);
                (param.ident.name, ty)
            })
            .collect();

        let return_ty = item
            .ret_type
            .as_ref()
            .map(|t| self.lower_ast_ty(t))
            .unwrap_or_else(InferTy::unit);

        self.clear_type_params();

        let sig = FnSig {
            type_params,
            params,
            return_ty,
        };

        self.register_function(name, sig);
    }

    /// Collect a function signature.
    fn collect_fn_signature(&mut self, fn_def: FnDef<'db>) {
        let ident = fn_def.ident(self.db());
        let name = ident.name;

        // TODO: Collect type parameters from fn_def when added to AST
        let type_params = vec![];

        // Set up type params context for parsing param types
        self.set_type_params(type_params.clone());

        // Convert parameter types
        let params: Vec<_> = fn_def
            .args(self.db())
            .iter()
            .map(|param| {
                let ty = self.lower_ast_ty(&param.ty);
                (param.ident.name, ty)
            })
            .collect();

        // Convert return type
        let return_ty = fn_def
            .ret_type(self.db())
            .as_ref()
            .map(|t| self.lower_ast_ty(t))
            .unwrap_or_else(InferTy::unit);

        self.clear_type_params();

        let sig = FnSig {
            type_params,
            params,
            return_ty,
        };

        self.register_function(name, sig);
    }

    /// Type check an item (second pass).
    fn check_item(&mut self, item: &Item<'db>) {
        match &item.kind {
            ItemKind::Fn(fn_def) => {
                self.check_function(*fn_def);
            }
            ItemKind::Struct(_struct_def) => {
                // TODO: Type check struct fields
            }
            ItemKind::Enum(_enum_def) => {
                // TODO: Type check enum variants
            }
            ItemKind::Module(module) => {
                // Recursively check items in submodule
                if let ModuleKind::Loaded(items, _, _) = module.kind(self.db()) {
                    for sub_item in items.iter() {
                        self.check_item(sub_item);
                    }
                }
            }
            ItemKind::ForeignMod(_) => {
                // Extern functions have no body to type check
            }
            ItemKind::Use(_) => {
                // Use statements don't need type checking
            }
            ItemKind::Impl(impl_block) => {
                for method in &impl_block.methods {
                    self.check_method(*method, impl_block.type_name.name);
                }
            }
        }
    }

    /// Collect a struct definition.
    fn collect_struct_definition(&mut self, struct_def: &StructDef<'db>) {
        let name = struct_def.ident.name;

        if let VariantData::Struct { fields } = &struct_def.data {
            let field_defs: Vec<_> = fields
                .iter()
                .filter_map(|field| {
                    let field_name = field.ident.as_ref()?.name;
                    let field_ty = self.lower_ast_ty(&field.ty);
                    Some((field_name, field_ty))
                })
                .collect();

            let def = crate::context::StructDef {
                type_params: vec![],
                fields: field_defs,
            };

            self.register_struct(name, def);
        }
    }

    /// Collect an enum definition.
    fn collect_enum_definition(&mut self, enum_def: &scrap_ast::enumdef::EnumDef<'db>) {
        let name = enum_def.ident.name;

        let variants: Vec<_> = enum_def
            .variants
            .iter()
            .map(|variant| {
                let variant_name = variant.ident.name;
                let data = match &variant.data {
                    VariantData::Unit(_) => EnumVariantDef::Unit,
                    VariantData::Tuple(fields, _) => {
                        let field_tys: Vec<_> = fields
                            .iter()
                            .map(|field| self.lower_ast_ty(&field.ty))
                            .collect();
                        EnumVariantDef::Tuple(field_tys)
                    }
                    VariantData::Struct { fields } => {
                        let field_defs: Vec<_> = fields
                            .iter()
                            .filter_map(|field| {
                                let field_name = field.ident.as_ref()?.name;
                                let field_ty = self.lower_ast_ty(&field.ty);
                                Some((field_name, field_ty))
                            })
                            .collect();
                        EnumVariantDef::Struct(field_defs)
                    }
                };
                (variant_name, data)
            })
            .collect();

        let def = crate::context::EnumDef {
            type_params: vec![],
            variants,
        };

        self.register_enum(name, def);
    }

    /// Type check a function body.
    fn check_function(&mut self, fn_def: FnDef<'db>) {
        let ident = fn_def.ident(self.db());
        let name = ident.name;

        let sig = match self.lookup_function(name) {
            Some(sig) => sig.clone(),
            None => return, // Should not happen if signature collection worked
        };

        // Set up function context
        self.push_scope();
        self.set_type_params(sig.type_params.clone());
        self.set_return_ty(sig.return_ty.clone());

        // Bind parameters
        for (param_name, param_ty) in &sig.params {
            self.define_var(*param_name, param_ty.clone());
        }

        // Type check body
        let body = fn_def.body(self.db());
        let body_ty = self.infer_block(body);

        if !sig.return_ty.is_unit() && !body_ty.is_never() {
            self.constrain_eq(
                body_ty.clone(),
                sig.return_ty.clone(),
                fn_def.span(self.db()),
            );
        }

        if body_ty.is_never() && fn_def.ret_type(self.db()).is_none() {
            self.record_fn_return_type(name, InferTy::Never);
            let mut updated_sig = sig;
            updated_sig.return_ty = InferTy::Never;
            self.register_function(name, updated_sig);
        }

        self.clear_return_ty();
        self.clear_type_params();
        self.pop_scope();
    }

    /// Collect signatures for all methods in an impl block.
    fn collect_impl_signatures(&mut self, impl_block: &ImplBlock<'db>) {
        let type_name = impl_block.type_name.name;

        for method in &impl_block.methods {
            let method_ident = method.ident(self.db());
            let mangled = Symbol::new(
                self.db(),
                format!(
                    "{}::{}",
                    type_name.text(self.db()),
                    method_ident.name.text(self.db())
                ),
            );

            let type_params = vec![];
            self.set_type_params(type_params.clone());

            let params: Vec<_> = method
                .args(self.db())
                .iter()
                .map(|param| {
                    let ty = self.lower_ast_ty(&param.ty);
                    (param.ident.name, ty)
                })
                .collect();

            let return_ty = method
                .ret_type(self.db())
                .as_ref()
                .map(|t| self.lower_ast_ty(t))
                .unwrap_or_else(InferTy::unit);

            self.clear_type_params();

            let sig = FnSig {
                type_params,
                params,
                return_ty,
            };

            self.register_function(mangled, sig);
        }
    }

    /// Type check a method body (same as check_function but uses mangled name).
    fn check_method(&mut self, fn_def: FnDef<'db>, type_name: Symbol<'db>) {
        let method_ident = fn_def.ident(self.db());
        let mangled = Symbol::new(
            self.db(),
            format!(
                "{}::{}",
                type_name.text(self.db()),
                method_ident.name.text(self.db())
            ),
        );

        let sig = match self.lookup_function(mangled) {
            Some(sig) => sig.clone(),
            None => return,
        };

        self.push_scope();
        self.set_type_params(sig.type_params.clone());
        self.set_return_ty(sig.return_ty.clone());

        for (param_name, param_ty) in &sig.params {
            self.define_var(*param_name, param_ty.clone());
        }

        let body = fn_def.body(self.db());
        let body_ty = self.infer_block(body);

        if !sig.return_ty.is_unit() && !body_ty.is_never() {
            self.constrain_eq(
                body_ty.clone(),
                sig.return_ty.clone(),
                fn_def.span(self.db()),
            );
        }

        if body_ty.is_never() && fn_def.ret_type(self.db()).is_none() {
            self.record_fn_return_type(mangled, InferTy::Never);
            let mut updated_sig = sig;
            updated_sig.return_ty = InferTy::Never;
            self.register_function(mangled, updated_sig);
        }

        self.clear_return_ty();
        self.clear_type_params();
        self.pop_scope();
    }
}
