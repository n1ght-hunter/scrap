//! Type checking context and environment.

use std::collections::HashMap;

use scrap_diagnostics::{AnnotationKind, Level, Snippet};
use scrap_errors::ErrorGuaranteed;
use scrap_shared::NodeId;
use scrap_shared::ident::Symbol;
use scrap_shared::types::{IntTy, Mutability};
use scrap_span::Span;

use crate::{
    constraints::{Constraint, ConstraintKind, ConstraintOrigin},
    resolved::ResolvedTy,
    types::{InferTy, TyVid},
};

/// Function signature for type checking.
#[derive(Debug, Clone)]
pub struct FnSig {
    /// Generic type parameters (e.g., `T` in `fn foo<T>`)
    pub type_params: Vec<Symbol>,
    /// Parameter names and types
    pub params: Vec<(Symbol, InferTy)>,
    /// Return type
    pub return_ty: InferTy,
}

/// Struct definition for type checking.
#[derive(Debug, Clone)]
pub struct StructDef {
    /// Generic type parameters
    pub type_params: Vec<Symbol>,
    /// Field names and types
    pub fields: Vec<(Symbol, InferTy)>,
}

/// An enum variant's data for type checking.
#[derive(Debug, Clone)]
pub enum EnumVariantDef {
    Unit,
    Tuple(Vec<InferTy>),
    Struct(Vec<(Symbol, InferTy)>),
}

/// Enum definition for type checking.
#[derive(Debug, Clone)]
pub struct EnumDef {
    /// Generic type parameters
    pub type_params: Vec<Symbol>,
    /// Variant names and their data
    pub variants: Vec<(Symbol, EnumVariantDef)>,
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
    ty_vars: Vec<Option<InferTy>>,

    /// Next type variable ID
    next_ty_vid: u32,

    /// Variable environment: name -> type
    /// Scoped via a stack of scopes
    scopes: Vec<HashMap<Symbol, InferTy>>,

    /// Variable mutability environment: name -> Mutability
    /// Scoped alongside the type scopes
    mutability_scopes: Vec<HashMap<Symbol, Mutability>>,

    /// Active borrows: variable name -> list of (Mutability, span)
    borrow_scopes: Vec<HashMap<Symbol, Vec<(Mutability, Span)>>>,

    /// Function signatures in scope
    functions: HashMap<Symbol, FnSig>,

    /// Struct definitions in scope
    structs: HashMap<Symbol, StructDef>,

    /// Enum definitions in scope
    enums: HashMap<Symbol, EnumDef>,

    /// Current function's return type (for checking return statements)
    current_return_ty: Option<InferTy>,

    /// Generic parameters in scope (for current function/struct)
    type_params: Vec<Symbol>,

    /// Collected constraints
    constraints: Vec<Constraint>,

    /// Recorded expression types during inference (NodeId -> InferTy)
    expr_types: HashMap<NodeId, InferTy>,

    /// Recorded local variable types (NodeId -> InferTy)
    local_types: HashMap<NodeId, InferTy>,

    /// Inferred function return types (function name -> InferTy)
    /// Populated during body checking when the inferred body type differs from the declared type.
    fn_return_types: HashMap<Symbol, InferTy>,

    /// Nesting depth of loops (for validating break/continue)
    pub loop_depth: usize,

    /// Generic function instantiations: (fn_name, call_node_id, type_param → concrete_type)
    generic_instantiations: Vec<(Symbol, NodeId, HashMap<Symbol, InferTy>)>,
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
            mutability_scopes: vec![HashMap::new()],
            borrow_scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_return_ty: None,
            type_params: Vec::new(),
            constraints: Vec::new(),
            expr_types: HashMap::new(),
            local_types: HashMap::new(),
            fn_return_types: HashMap::new(),
            loop_depth: 0,
            generic_instantiations: Vec::new(),
        }
    }

    /// Get the database reference.
    pub fn db(&self) -> &'db dyn scrap_shared::Db {
        self.db
    }

    /// Create a fresh type variable.
    pub fn fresh_ty_var(&mut self) -> InferTy {
        let vid = TyVid(self.next_ty_vid);
        self.next_ty_vid += 1;
        self.ty_vars.push(None);
        InferTy::Var(vid)
    }

    /// Get the current binding of a type variable (if solved).
    pub fn probe(&self, vid: TyVid) -> Option<&InferTy> {
        self.ty_vars
            .get(vid.0 as usize)
            .and_then(|opt| opt.as_ref())
    }

    /// Bind a type variable to a type.
    pub fn bind(&mut self, vid: TyVid, ty: InferTy) {
        if let Some(slot) = self.ty_vars.get_mut(vid.0 as usize) {
            *slot = Some(ty);
        }
    }

    /// Resolve a type, following type variable chains.
    pub fn resolve(&self, ty: &InferTy) -> InferTy {
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

    /// Push a new scope onto the scope stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.mutability_scopes.push(HashMap::new());
        self.borrow_scopes.push(HashMap::new());
    }

    /// Pop the current scope from the scope stack.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
        self.mutability_scopes.pop();
        self.borrow_scopes.pop();
    }

    /// Define a variable in the current scope (immutable by default).
    pub fn define_var(&mut self, name: Symbol, ty: InferTy) {
        self.define_var_with_mutability(name, ty, Mutability::Not);
    }

    /// Define a variable in the current scope with explicit mutability.
    pub fn define_var_with_mutability(
        &mut self,
        name: Symbol,
        ty: InferTy,
        mutability: Mutability,
    ) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
        if let Some(scope) = self.mutability_scopes.last_mut() {
            scope.insert(name, mutability);
        }
    }

    /// Look up a variable in all scopes (innermost first).
    pub fn lookup_var(&self, name: Symbol) -> Option<InferTy> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(&name) {
                return Some(ty.clone());
            }
        }
        None
    }

    /// Look up a variable's mutability in all scopes (innermost first).
    pub fn lookup_var_mutability(&self, name: Symbol) -> Option<Mutability> {
        for scope in self.mutability_scopes.iter().rev() {
            if let Some(m) = scope.get(&name) {
                return Some(*m);
            }
        }
        None
    }

    /// Record a borrow of a variable and check borrow rules.
    pub fn record_borrow(&mut self, name: Symbol, mutability: Mutability, span: Span) {
        // Collect existing borrows across all scopes
        let existing: Vec<(Mutability, Span)> = self
            .borrow_scopes
            .iter()
            .filter_map(|scope| scope.get(&name))
            .flat_map(|borrows| borrows.iter().copied())
            .collect();

        if mutability.is_mut() {
            // &mut: no other borrows allowed
            if !existing.is_empty() {
                self.emit_borrow_conflict(name.text(), span);
            }
        } else if existing.iter().any(|(m, _)| m.is_mut()) {
            self.emit_borrow_conflict(name.text(), span);
        }

        // Record the borrow in the current scope
        if let Some(scope) = self.borrow_scopes.last_mut() {
            scope
                .entry(name)
                .or_insert_with(Vec::new)
                .push((mutability, span));
        }
    }

    /// Look up the recorded InferTy for a given expression NodeId.
    pub fn lookup_expr_type_infer(&self, node_id: NodeId) -> InferTy {
        self.expr_types
            .get(&node_id)
            .cloned()
            .unwrap_or(InferTy::Error)
    }

    /// Set the type parameters for the current context.
    pub fn set_type_params(&mut self, params: Vec<Symbol>) {
        self.type_params = params;
    }

    /// Clear the type parameters.
    pub fn clear_type_params(&mut self) {
        self.type_params.clear();
    }

    /// Check if a symbol is a type parameter.
    pub fn is_type_param(&self, name: Symbol) -> bool {
        self.type_params.contains(&name)
    }

    /// Get the type parameters.
    pub fn type_params(&self) -> &[Symbol] {
        &self.type_params
    }

    /// Set the current function's return type.
    pub fn set_return_ty(&mut self, ty: InferTy) {
        self.current_return_ty = Some(ty);
    }

    /// Clear the current function's return type.
    pub fn clear_return_ty(&mut self) {
        self.current_return_ty = None;
    }

    /// Get the current function's return type.
    pub fn return_ty(&self) -> Option<&InferTy> {
        self.current_return_ty.as_ref()
    }

    /// Register a function signature.
    pub fn register_function(&mut self, name: Symbol, sig: FnSig) {
        self.functions.insert(name, sig);
    }

    /// Look up a function signature.
    pub fn lookup_function(&self, name: Symbol) -> Option<&FnSig> {
        self.functions.get(&name)
    }

    /// Register a struct definition.
    pub fn register_struct(&mut self, name: Symbol, def: StructDef) {
        self.structs.insert(name, def);
    }

    /// Look up a struct definition.
    pub fn lookup_struct(&self, name: Symbol) -> Option<&StructDef> {
        self.structs.get(&name)
    }

    /// Register an enum definition.
    pub fn register_enum(&mut self, name: Symbol, def: EnumDef) {
        self.enums.insert(name, def);
    }

    /// Look up an enum definition.
    pub fn lookup_enum(&self, name: Symbol) -> Option<&EnumDef> {
        self.enums.get(&name)
    }

    /// Add an equality constraint between two types.
    pub fn constrain_eq(&mut self, t1: InferTy, t2: InferTy, span: Span) {
        let origin = ConstraintOrigin::new(span, ConstraintKind::Assignment);
        self.constraints.push(Constraint::Eq(t1, t2, origin));
    }

    /// Add an equality constraint with a specific origin kind.
    pub fn constrain_eq_with_kind(
        &mut self,
        t1: InferTy,
        t2: InferTy,
        span: Span,
        kind: ConstraintKind,
    ) {
        let origin = ConstraintOrigin::new(span, kind);
        self.constraints.push(Constraint::Eq(t1, t2, origin));
    }

    /// Take all collected constraints (draining them).
    pub fn take_constraints(&mut self) -> Vec<Constraint> {
        std::mem::take(&mut self.constraints)
    }

    /// Record the type of an expression.
    pub fn record_expr_type(&mut self, node_id: NodeId, ty: InferTy) {
        self.expr_types.insert(node_id, ty);
    }

    /// Record the type of a local variable.
    pub fn record_local_type(&mut self, node_id: NodeId, ty: InferTy) {
        self.local_types.insert(node_id, ty);
    }

    /// Record the inferred return type of a function.
    pub fn record_fn_return_type(&mut self, name: Symbol, ty: InferTy) {
        self.fn_return_types.insert(name, ty);
    }

    /// Finalize all recorded types after unification.
    /// Converts InferTy to ResolvedTy by resolving all type variables.
    #[allow(clippy::type_complexity)]
    pub fn finalize_types(
        &self,
    ) -> (
        hashbrown::HashMap<scrap_shared::NodeId, ResolvedTy>,
        hashbrown::HashMap<scrap_shared::NodeId, ResolvedTy>,
        hashbrown::HashMap<Symbol, ResolvedTy>,
        hashbrown::HashMap<Symbol, Vec<(NodeId, Vec<(Symbol, ResolvedTy)>)>>,
    ) {
        let expr_types = self
            .expr_types
            .iter()
            .map(|(id, ty)| (*id, self.resolve_to_final(ty)))
            .collect();

        let local_types = self
            .local_types
            .iter()
            .map(|(id, ty)| (*id, self.resolve_to_final(ty)))
            .collect();

        let fn_return_types = self
            .fn_return_types
            .iter()
            .map(|(name, ty)| (*name, self.resolve_to_final(ty)))
            .collect();

        let mut generic_instantiations: hashbrown::HashMap<Symbol, Vec<_>> =
            hashbrown::HashMap::new();
        for (name, node_id, subst) in &self.generic_instantiations {
            let resolved_subst: Vec<_> = subst
                .iter()
                .map(|(param, ty)| (*param, self.resolve_to_final(ty)))
                .collect();
            generic_instantiations
                .entry(*name)
                .or_default()
                .push((*node_id, resolved_subst));
        }

        (
            expr_types,
            local_types,
            fn_return_types,
            generic_instantiations,
        )
    }

    /// Convert InferTy to ResolvedTy after solving constraints.
    fn resolve_to_final(&self, ty: &InferTy) -> ResolvedTy {
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
            InferTy::Ref(inner, m) => ResolvedTy::Ref(Box::new(self.resolve_to_final(&inner)), m),
            InferTy::Ptr(inner) => ResolvedTy::Ptr(Box::new(self.resolve_to_final(&inner))),
            InferTy::Error => ResolvedTy::Error,
        }
    }

    /// Emit a type mismatch error.
    pub fn emit_type_mismatch(&self, expected: &str, found: &str, span: Span) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR.primary_title("type mismatch").element(
                Snippet::source(self.source)
                    .path(self.file_name)
                    .annotation(
                        AnnotationKind::Primary
                            .span(span.range())
                            .label(format!("expected `{}`, found `{}`", expected, found)),
                    ),
            ),
        )
    }

    pub fn record_generic_instantiation(
        &mut self,
        fn_name: Symbol,
        call_site: NodeId,
        subst: HashMap<Symbol, InferTy>,
    ) {
        self.generic_instantiations
            .push((fn_name, call_site, subst));
    }

    pub fn emit_undefined_type(&self, name: &str, span: Span) -> ErrorGuaranteed {
        let type_params = self.type_params();
        let suggestion = type_params
            .iter()
            .find(|p| {
                let p_text = p.text();
                p_text.len() == 1 && name.len() == 1 || p_text.to_lowercase() == name.to_lowercase()
            })
            .map(|p| p.text().to_string());

        let mut group = Level::ERROR
            .primary_title(format!("undefined type `{}`", name))
            .element(
                Snippet::source(self.source)
                    .path(self.file_name)
                    .annotation(AnnotationKind::Primary.span(span.range())),
            );

        if let Some(suggested) = suggestion {
            group = group.element(
                Level::HELP.message(format!("did you mean the type parameter `{}`?", suggested)),
            );
        } else if !type_params.is_empty() {
            let params: Vec<_> = type_params.iter().map(|p| p.text().to_string()).collect();
            group = group.element(
                Level::NOTE.message(format!("available type parameters: {}", params.join(", "))),
            );
        }

        self.db.dcx().emit_err(group)
    }

    pub fn emit_error(&self, msg: &str, span: Span) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR.primary_title(msg).element(
                Snippet::source(self.source)
                    .path(self.file_name)
                    .annotation(AnnotationKind::Primary.span(span.range())),
            ),
        )
    }

    /// Emit an error for assignment to an immutable variable.
    pub fn emit_immutable_assign_error(&self, name: &str, span: Span) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!("cannot assign to immutable variable `{}`", name))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.range())
                                .label("cannot assign"),
                        ),
                )
                .element(Level::NOTE.message(format!(
                    "consider making this binding mutable: `let mut {}`",
                    name
                ))),
        )
    }

    /// Emit an error for borrowing an immutable variable as mutable.
    pub fn emit_cannot_borrow_as_mutable(&self, name: &str, span: Span) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!(
                    "cannot borrow `{}` as mutable, as it is not declared as mutable",
                    name
                ))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.range())
                                .label("cannot borrow as mutable"),
                        ),
                )
                .element(Level::NOTE.message(format!(
                    "consider making this binding mutable: `let mut {}`",
                    name
                ))),
        )
    }

    /// Emit a borrow conflict error.
    pub fn emit_borrow_conflict(&self, name: &str, span: Span) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!(
                    "cannot borrow `{}` because it is already borrowed",
                    name
                ))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.range())
                                .label("conflicting borrow"),
                        ),
                ),
        )
    }

    /// Emit an error for writing through an immutable reference.
    pub fn emit_immutable_ref_deref_error(&self, span: Span) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title("cannot assign to data behind a `&` reference")
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.range())
                                .label("cannot assign through `&` reference"),
                        ),
                ),
        )
    }

    /// Emit an undefined variable error.
    pub fn emit_undefined_variable(&self, name: &str, span: Span) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!("undefined variable `{}`", name))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.range())
                                .label("not found in this scope"),
                        ),
                ),
        )
    }

    /// Emit an undefined function error.
    pub fn emit_undefined_function(&self, name: &str, span: Span) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!("undefined function `{}`", name))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.range())
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
        span: Span,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title("wrong number of arguments")
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.range())
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
        field_span: Span,
        note: String,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!(
                    "struct `{struct_name}` has no field named `{field_name}`"
                ))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(field_span.range())
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
        span: Span,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title(format!(
                    "missing field `{field_name}` in initializer of `{struct_name}`"
                ))
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(
                            AnnotationKind::Primary
                                .span(span.range())
                                .label(format!("field `{field_name}` is missing")),
                        ),
                ),
        )
    }

    /// Emit an infinite type error (occurs check failure).
    pub fn emit_infinite_type(&self, var_name: &str, ty: &str, span: Span) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR.primary_title("infinite type").element(
                Snippet::source(self.source)
                    .path(self.file_name)
                    .annotation(
                        AnnotationKind::Primary
                            .span(span.range())
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
        span: Span,
    ) -> ErrorGuaranteed {
        self.db.dcx().emit_err(
            Level::ERROR
                .primary_title("wrong number of type arguments")
                .element(
                    Snippet::source(self.source)
                        .path(self.file_name)
                        .annotation(AnnotationKind::Primary.span(span.range()).label(format!(
                            "expected {} type arguments, found {}",
                            expected, found
                        ))),
                ),
        )
    }

    /// Convert a type to a human-readable string.
    pub fn ty_to_string(&self, ty: &InferTy) -> String {
        let resolved = self.resolve(ty);
        self.ty_to_string_inner(&resolved)
    }

    fn ty_to_string_inner(&self, ty: &InferTy) -> String {
        match ty {
            InferTy::Var(vid) => format!("?{}", vid.0),
            InferTy::Void => "void".to_string(),
            InferTy::Bool => "bool".to_string(),
            InferTy::Int(k) => k.name_str().to_string(),
            InferTy::Uint(k) => k.name_str().to_string(),
            InferTy::Float(k) => k.name_str().to_string(),
            InferTy::Str => "String".to_string(),
            InferTy::Never => "!".to_string(),
            InferTy::Adt(name) => name.text().to_string(),
            InferTy::Param(name) => name.text().to_string(),
            InferTy::App(name, args) => {
                let args_str: Vec<_> = args.iter().map(|a| self.ty_to_string_inner(a)).collect();
                format!("{}<{}>", name.text(), args_str.join(", "))
            }
            InferTy::Fn(params, ret) => {
                let params_str: Vec<_> =
                    params.iter().map(|p| self.ty_to_string_inner(p)).collect();
                format!(
                    "fn({}) -> {}",
                    params_str.join(", "),
                    self.ty_to_string_inner(ret)
                )
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
