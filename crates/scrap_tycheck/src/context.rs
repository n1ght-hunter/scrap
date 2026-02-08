//! Type checking context and environment.

use std::collections::HashMap;

use scrap_diagnostics::{AnnotationKind, Level, Snippet};
use scrap_errors::ErrorGuaranteed;
use scrap_shared::ident::Symbol;
use scrap_shared::types::IntTy;
use scrap_shared::NodeId;
use scrap_span::Span;

use crate::{
    constraints::{Constraint, ConstraintKind, ConstraintOrigin},
    resolved::ResolvedTy,
    types::{InferTy, TyVid},
};

/// Function signature for type checking.
#[derive(Debug, Clone)]
pub struct FnSig<'db> {
    /// Generic type parameters (e.g., `T` in `fn foo<T>`)
    pub type_params: Vec<Symbol<'db>>,
    /// Parameter names and types
    pub params: Vec<(Symbol<'db>, InferTy<'db>)>,
    /// Return type
    pub return_ty: InferTy<'db>,
}

/// Struct definition for type checking.
#[derive(Debug, Clone)]
pub struct StructDef<'db> {
    /// Generic type parameters
    pub type_params: Vec<Symbol<'db>>,
    /// Field names and types
    pub fields: Vec<(Symbol<'db>, InferTy<'db>)>,
}

/// Enum definition for type checking.
#[derive(Debug, Clone)]
pub struct EnumDef<'db> {
    /// Generic type parameters
    pub type_params: Vec<Symbol<'db>>,
    /// Variant names and their field types (if any)
    pub variants: Vec<(Symbol<'db>, Vec<InferTy<'db>>)>,
}

/// The type checking context.
/// Maintains all state needed during type checking and inference.
pub struct TypeContext<'db> {
    /// Reference to the salsa database
    db: &'db dyn scrap_shared::Db,

    /// Source code being type checked (for error messages)
    source: &'db str,

    /// File name (for error messages)
    file_name: &'db str,

    /// Type variable storage: TyVid -> Option<InferTy>
    /// None = unsolved, Some = solved
    ty_vars: Vec<Option<InferTy<'db>>>,

    /// Next type variable ID
    next_ty_vid: u32,

    /// Variable environment: name -> type
    /// Scoped via a stack of scopes
    scopes: Vec<HashMap<Symbol<'db>, InferTy<'db>>>,

    /// Function signatures in scope
    functions: HashMap<Symbol<'db>, FnSig<'db>>,

    /// Struct definitions in scope
    structs: HashMap<Symbol<'db>, StructDef<'db>>,

    /// Enum definitions in scope
    enums: HashMap<Symbol<'db>, EnumDef<'db>>,

    /// Current function's return type (for checking return statements)
    current_return_ty: Option<InferTy<'db>>,

    /// Generic parameters in scope (for current function/struct)
    type_params: Vec<Symbol<'db>>,

    /// Collected constraints
    constraints: Vec<Constraint<'db>>,

    /// Recorded expression types during inference (NodeId -> InferTy)
    expr_types: HashMap<NodeId, InferTy<'db>>,

    /// Recorded local variable types (NodeId -> InferTy)
    local_types: HashMap<NodeId, InferTy<'db>>,

    /// Inferred function return types (function name -> InferTy)
    /// Populated during body checking when the inferred body type differs from the declared type.
    fn_return_types: HashMap<Symbol<'db>, InferTy<'db>>,
}

impl<'db> TypeContext<'db> {
    /// Create a new type checking context.
    pub fn new(db: &'db dyn scrap_shared::Db, source: &'db str, file_name: &'db str) -> Self {
        Self {
            db,
            source,
            file_name,
            ty_vars: Vec::new(),
            next_ty_vid: 0,
            scopes: vec![HashMap::new()], // Global scope
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_return_ty: None,
            type_params: Vec::new(),
            constraints: Vec::new(),
            expr_types: HashMap::new(),
            local_types: HashMap::new(),
            fn_return_types: HashMap::new(),
        }
    }

    /// Get the database reference.
    pub fn db(&self) -> &'db dyn scrap_shared::Db {
        self.db
    }

    // === Type Variable Management ===

    /// Create a fresh type variable.
    pub fn fresh_ty_var(&mut self) -> InferTy<'db> {
        let vid = TyVid(self.next_ty_vid);
        self.next_ty_vid += 1;
        self.ty_vars.push(None);
        InferTy::Var(vid)
    }

    /// Get the current binding of a type variable (if solved).
    pub fn probe(&self, vid: TyVid) -> Option<&InferTy<'db>> {
        self.ty_vars.get(vid.0 as usize).and_then(|opt| opt.as_ref())
    }

    /// Bind a type variable to a type.
    pub fn bind(&mut self, vid: TyVid, ty: InferTy<'db>) {
        if let Some(slot) = self.ty_vars.get_mut(vid.0 as usize) {
            *slot = Some(ty);
        }
    }

    /// Resolve a type, following type variable chains.
    pub fn resolve(&self, ty: &InferTy<'db>) -> InferTy<'db> {
        match ty {
            InferTy::Var(vid) => {
                if let Some(resolved) = self.probe(*vid) {
                    self.resolve(resolved)
                } else {
                    ty.clone()
                }
            }
            // Recursively resolve nested types
            InferTy::App(name, args) => {
                let resolved_args: Vec<_> = args.iter().map(|a| self.resolve(a)).collect();
                InferTy::App(*name, resolved_args)
            }
            InferTy::Fn(params, ret) => {
                let resolved_params: Vec<_> = params.iter().map(|p| self.resolve(p)).collect();
                let resolved_ret = self.resolve(ret);
                InferTy::Fn(resolved_params, Box::new(resolved_ret))
            }
            InferTy::Tuple(elems) => {
                let resolved: Vec<_> = elems.iter().map(|e| self.resolve(e)).collect();
                InferTy::Tuple(resolved)
            }
            InferTy::Ref(inner, m) => {
                let resolved_inner = self.resolve(inner);
                InferTy::Ref(Box::new(resolved_inner), *m)
            }
            InferTy::Ptr(inner) => {
                let resolved_inner = self.resolve(inner);
                InferTy::Ptr(Box::new(resolved_inner))
            }
            _ => ty.clone(),
        }
    }

    // === Scope Management ===

    /// Push a new scope onto the scope stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope from the scope stack.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a variable in the current scope.
    pub fn define_var(&mut self, name: Symbol<'db>, ty: InferTy<'db>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    /// Look up a variable in all scopes (innermost first).
    pub fn lookup_var(&self, name: Symbol<'db>) -> Option<InferTy<'db>> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(&name) {
                return Some(ty.clone());
            }
        }
        None
    }

    // === Type Parameters ===

    /// Set the type parameters for the current context.
    pub fn set_type_params(&mut self, params: Vec<Symbol<'db>>) {
        self.type_params = params;
    }

    /// Clear the type parameters.
    pub fn clear_type_params(&mut self) {
        self.type_params.clear();
    }

    /// Check if a symbol is a type parameter.
    pub fn is_type_param(&self, name: Symbol<'db>) -> bool {
        self.type_params.contains(&name)
    }

    /// Get the type parameters.
    pub fn type_params(&self) -> &[Symbol<'db>] {
        &self.type_params
    }

    // === Return Type ===

    /// Set the current function's return type.
    pub fn set_return_ty(&mut self, ty: InferTy<'db>) {
        self.current_return_ty = Some(ty);
    }

    /// Clear the current function's return type.
    pub fn clear_return_ty(&mut self) {
        self.current_return_ty = None;
    }

    /// Get the current function's return type.
    pub fn return_ty(&self) -> Option<&InferTy<'db>> {
        self.current_return_ty.as_ref()
    }

    // === Function Management ===

    /// Register a function signature.
    pub fn register_function(&mut self, name: Symbol<'db>, sig: FnSig<'db>) {
        self.functions.insert(name, sig);
    }

    /// Look up a function signature.
    pub fn lookup_function(&self, name: Symbol<'db>) -> Option<&FnSig<'db>> {
        self.functions.get(&name)
    }

    // === Struct Management ===

    /// Register a struct definition.
    pub fn register_struct(&mut self, name: Symbol<'db>, def: StructDef<'db>) {
        self.structs.insert(name, def);
    }

    /// Look up a struct definition.
    pub fn lookup_struct(&self, name: Symbol<'db>) -> Option<&StructDef<'db>> {
        self.structs.get(&name)
    }

    // === Enum Management ===

    /// Register an enum definition.
    pub fn register_enum(&mut self, name: Symbol<'db>, def: EnumDef<'db>) {
        self.enums.insert(name, def);
    }

    /// Look up an enum definition.
    pub fn lookup_enum(&self, name: Symbol<'db>) -> Option<&EnumDef<'db>> {
        self.enums.get(&name)
    }

    // === Constraint Collection ===

    /// Add an equality constraint between two types.
    pub fn constrain_eq(&mut self, t1: InferTy<'db>, t2: InferTy<'db>, span: Span<'db>) {
        let origin = ConstraintOrigin::new(span, ConstraintKind::Assignment);
        self.constraints.push(Constraint::Eq(t1, t2, origin));
    }

    /// Add an equality constraint with a specific origin kind.
    pub fn constrain_eq_with_kind(
        &mut self,
        t1: InferTy<'db>,
        t2: InferTy<'db>,
        span: Span<'db>,
        kind: ConstraintKind,
    ) {
        let origin = ConstraintOrigin::new(span, kind);
        self.constraints.push(Constraint::Eq(t1, t2, origin));
    }

    /// Take all collected constraints (draining them).
    pub fn take_constraints(&mut self) -> Vec<Constraint<'db>> {
        std::mem::take(&mut self.constraints)
    }

    // === Type Recording ===

    /// Record the type of an expression.
    pub fn record_expr_type(&mut self, node_id: NodeId, ty: InferTy<'db>) {
        self.expr_types.insert(node_id, ty);
    }

    /// Record the type of a local variable.
    pub fn record_local_type(&mut self, node_id: NodeId, ty: InferTy<'db>) {
        self.local_types.insert(node_id, ty);
    }

    /// Record the inferred return type of a function.
    pub fn record_fn_return_type(&mut self, name: Symbol<'db>, ty: InferTy<'db>) {
        self.fn_return_types.insert(name, ty);
    }

    /// Finalize all recorded types after unification.
    /// Converts InferTy to ResolvedTy by resolving all type variables.
    /// Returns (expr_types, local_types, fn_return_types) Vecs for creating a TypeTable.
    pub fn finalize_types(
        &self,
    ) -> (
        Vec<(scrap_shared::NodeId, ResolvedTy<'db>)>,
        Vec<(scrap_shared::NodeId, ResolvedTy<'db>)>,
        Vec<(Symbol<'db>, ResolvedTy<'db>)>,
    ) {
        let expr_types: Vec<_> = self
            .expr_types
            .iter()
            .map(|(id, ty)| (*id, self.resolve_to_final(ty)))
            .collect();

        let local_types: Vec<_> = self
            .local_types
            .iter()
            .map(|(id, ty)| (*id, self.resolve_to_final(ty)))
            .collect();

        let fn_return_types: Vec<_> = self
            .fn_return_types
            .iter()
            .map(|(name, ty)| (*name, self.resolve_to_final(ty)))
            .collect();

        (expr_types, local_types, fn_return_types)
    }

    /// Convert InferTy to ResolvedTy after solving constraints.
    fn resolve_to_final(&self, ty: &InferTy<'db>) -> ResolvedTy<'db> {
        let resolved = self.resolve(ty);
        match resolved {
            InferTy::Var(_) => ResolvedTy::Int(IntTy::I32), // Unsolved variable defaults to i32
            InferTy::Void => ResolvedTy::Void,
            InferTy::Bool => ResolvedTy::Bool,
            InferTy::Int(k) => ResolvedTy::Int(k),
            InferTy::Uint(k) => ResolvedTy::Uint(k),
            InferTy::Float(k) => ResolvedTy::Float(k),
            InferTy::Str => ResolvedTy::Str,
            InferTy::Never => ResolvedTy::Never,
            InferTy::Adt(s) => ResolvedTy::Adt(s),
            InferTy::Param(s) => ResolvedTy::Param(s),
            InferTy::App(n, args) => {
                ResolvedTy::App(n, args.iter().map(|a| self.resolve_to_final(a)).collect())
            }
            InferTy::Fn(params, ret) => ResolvedTy::Fn(
                params.iter().map(|p| self.resolve_to_final(p)).collect(),
                Box::new(self.resolve_to_final(&ret)),
            ),
            InferTy::Tuple(elems) => {
                ResolvedTy::Tuple(elems.iter().map(|e| self.resolve_to_final(e)).collect())
            }
            InferTy::Ref(inner, m) => {
                ResolvedTy::Ref(Box::new(self.resolve_to_final(&inner)), m)
            }
            InferTy::Ptr(inner) => {
                ResolvedTy::Ptr(Box::new(self.resolve_to_final(&inner)))
            }
            InferTy::Error => ResolvedTy::Error,
        }
    }

    // === Error Emission ===

    /// Emit a type mismatch error.
    pub fn emit_type_mismatch(
        &self,
        expected: &str,
        found: &str,
        span: Span<'db>,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title("type mismatch")
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.to_range(self.db))
                                .label(format!("expected `{}`, found `{}`", expected, found)),
                        ),
                ),
        )
    }

    /// Emit an undefined variable error.
    pub fn emit_undefined_variable(&self, name: &str, span: Span<'db>) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(&format!("undefined variable `{}`", name))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.to_range(self.db))
                                .label("not found in this scope"),
                        ),
                ),
        )
    }

    /// Emit an undefined function error.
    pub fn emit_undefined_function(&self, name: &str, span: Span<'db>) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(&format!("undefined function `{}`", name))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.to_range(self.db))
                                .label("not found in this scope"),
                        ),
                ),
        )
    }

    /// Emit an arity mismatch error.
    pub fn emit_arity_mismatch(
        &self,
        expected: usize,
        found: usize,
        span: Span<'db>,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title("wrong number of arguments")
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.to_range(self.db))
                                .label(format!("expected {} arguments, found {}", expected, found)),
                        ),
                ),
        )
    }

    /// Emit an error for an unknown field in a struct initializer.
    pub fn emit_unknown_struct_field(
        &self,
        struct_name: &str,
        field_name: &str,
        field_span: Span<'db>,
        note: String,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!("struct `{struct_name}` has no field named `{field_name}`"))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(field_span.to_range(self.db))
                                .label(format!("`{struct_name}` does not have this field")),
                        ),
                )
                .element(Level::NOTE.message(note)),
        )
    }

    /// Emit an error for a missing field in a struct initializer.
    pub fn emit_missing_struct_field(
        &self,
        struct_name: &str,
        field_name: &str,
        span: Span<'db>,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!("missing field `{field_name}` in initializer of `{struct_name}`"))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.to_range(self.db))
                                .label(format!("field `{field_name}` is missing")),
                        ),
                ),
        )
    }

    /// Emit an infinite type error (occurs check failure).
    pub fn emit_infinite_type(
        &self,
        var_name: &str,
        ty: &str,
        span: Span<'db>,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title("infinite type")
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.to_range(self.db))
                                .label(format!("`{}` occurs in `{}`", var_name, ty)),
                        ),
                ),
        )
    }

    /// Emit a type arity mismatch error.
    pub fn emit_type_arity_mismatch(
        &self,
        expected: usize,
        found: usize,
        span: Span<'db>,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title("wrong number of type arguments")
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.to_range(self.db))
                                .label(format!(
                                    "expected {} type arguments, found {}",
                                    expected, found
                                )),
                        ),
                ),
        )
    }

    // === Type Display ===

    /// Convert a type to a human-readable string.
    pub fn ty_to_string(&self, ty: &InferTy<'db>) -> String {
        let resolved = self.resolve(ty);
        self.ty_to_string_inner(&resolved)
    }

    fn ty_to_string_inner(&self, ty: &InferTy<'db>) -> String {
        match ty {
            InferTy::Var(vid) => format!("?{}", vid.0),
            InferTy::Void => "void".to_string(),
            InferTy::Bool => "bool".to_string(),
            InferTy::Int(k) => k.name_str().to_string(),
            InferTy::Uint(k) => k.name_str().to_string(),
            InferTy::Float(k) => k.name_str().to_string(),
            InferTy::Str => "String".to_string(),
            InferTy::Never => "!".to_string(),
            InferTy::Adt(name) => name.text(self.db).to_string(),
            InferTy::Param(name) => name.text(self.db).to_string(),
            InferTy::App(name, args) => {
                let args_str: Vec<_> = args.iter().map(|a| self.ty_to_string_inner(a)).collect();
                format!("{}<{}>", name.text(self.db), args_str.join(", "))
            }
            InferTy::Fn(params, ret) => {
                let params_str: Vec<_> =
                    params.iter().map(|p| self.ty_to_string_inner(p)).collect();
                format!("fn({}) -> {}", params_str.join(", "), self.ty_to_string_inner(ret))
            }
            InferTy::Tuple(elems) => {
                if elems.is_empty() {
                    "()".to_string()
                } else {
                    let elems_str: Vec<_> =
                        elems.iter().map(|e| self.ty_to_string_inner(e)).collect();
                    format!("({})", elems_str.join(", "))
                }
            }
            InferTy::Ref(inner, m) => {
                format!("{}{}", m.ref_prefix_str(), self.ty_to_string_inner(inner))
            }
            InferTy::Ptr(inner) => {
                format!("*{}", self.ty_to_string_inner(inner))
            }
            InferTy::Error => "<error>".to_string(),
        }
    }
}
