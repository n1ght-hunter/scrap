//! Top-level type checking for modules and items.

use scrap_ast::{
    fndef::FnDef,
    foreign::ForeignItem,
    item::{Item, ItemKind},
    module::ModuleKind,
    Can,
};

use crate::{
    context::{FnSig, TypeContext},
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
            ItemKind::Struct(_struct_def) => {
                // TODO: Collect struct definition
            }
            ItemKind::Enum(_enum_def) => {
                // TODO: Collect enum definition
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
        }
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
        let body_ty = self.infer_block(&body);

        // If the function has a non-unit return type and the body doesn't end
        // with a return statement, the body's type should match the return type
        if !sig.return_ty.is_unit() && !body_ty.is_never() {
            self.constrain_eq(body_ty, sig.return_ty.clone(), fn_def.span(self.db()));
        }

        // Clean up
        self.clear_return_ty();
        self.clear_type_params();
        self.pop_scope();
    }
}
