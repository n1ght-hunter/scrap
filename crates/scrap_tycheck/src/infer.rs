//! Type inference for expressions.

use std::collections::HashMap;

use scrap_ast::{
    block::Block,
    expr::{Expr, ExprKind},
    lit::{Lit, LitKind},
    local::{Local, LocalKind},
    operators::{AssignOp, BinOp, BinOpKind, UnOp},
    pat::PatKind,
    stmt::{Stmt, StmtKind},
    typedef::{Ty, TyKind},
};
use scrap_shared::types::{FloatTy, IntTy, Mutability, UintTy};
use scrap_shared::{ident::Symbol, path::Path};
use scrap_span::Span;

use crate::{constraints::ConstraintKind, context::TypeContext, types::InferTy};

impl<'db> TypeContext<'db> {
    /// Infer the type of an expression.
    pub fn infer_expr(&mut self, expr: &Expr<'db>) -> InferTy<'db> {
        let ty = match &expr.kind {
            ExprKind::Lit(lit) => self.infer_literal(lit),

            ExprKind::Path(path) => self.infer_path(path, expr.span),

            ExprKind::Binary(op, lhs, rhs) => self.infer_binary_op(op, lhs, rhs, expr.span),

            ExprKind::Call(callee, args) => self.infer_call(callee, args, expr.span),

            ExprKind::If(cond, then_block, else_expr) => {
                self.infer_if(cond, then_block, else_expr.as_deref(), expr.span)
            }

            ExprKind::Block(block) => self.infer_block(block),

            ExprKind::Array(elements) => self.infer_array(elements, expr.span),

            ExprKind::Paren(inner) => self.infer_expr(inner),

            ExprKind::Return(value) => self.infer_return(value.as_deref(), expr.span),

            ExprKind::Assign(lhs, rhs, _) => self.infer_assign(lhs, rhs, expr.span),

            ExprKind::AssignOp(op, lhs, rhs) => self.infer_assign_op(op, lhs, rhs, expr.span),

            ExprKind::Unary(op, inner) => self.infer_unary_op(*op, inner, expr.span),

            ExprKind::Struct(struct_expr) => {
                self.infer_struct_init(&struct_expr.path, &struct_expr.fields, expr.span)
            }

            ExprKind::Field(base, field_ident) => {
                self.infer_field_access(base, field_ident, expr.span)
            }

            ExprKind::Match(scrutinee, arms) => self.infer_match(scrutinee, arms, expr.span),

            ExprKind::MethodCall(receiver, method, args) => {
                self.infer_method_call(receiver, method, args, expr.span)
            }

            ExprKind::AddrOf(mutability, inner) => {
                self.infer_addr_of(*mutability, inner, expr.span)
            }

            ExprKind::Spawn(inner) => {
                self.infer_expr(inner);
                InferTy::Void
            }

            ExprKind::Loop(block) => {
                self.loop_depth += 1;
                self.infer_block(block);
                self.loop_depth -= 1;
                InferTy::Never
            }

            ExprKind::While(cond, block) => {
                let cond_ty = self.infer_expr(cond);
                self.constrain_eq(cond_ty, InferTy::Bool, cond.span);
                self.loop_depth += 1;
                self.infer_block(block);
                self.loop_depth -= 1;
                InferTy::Void
            }

            ExprKind::Break(value) => {
                if self.loop_depth == 0 {
                    self.emit_error("break outside of loop", expr.span);
                }
                if let Some(val) = value {
                    self.infer_expr(val);
                }
                InferTy::Never
            }

            ExprKind::Continue => {
                if self.loop_depth == 0 {
                    self.emit_error("continue outside of loop", expr.span);
                }
                InferTy::Never
            }

            ExprKind::Err => InferTy::Error,
        };

        // Record the type for this expression
        self.record_expr_type(expr.id, ty.clone());

        ty
    }

    /// Infer the type of a literal.
    fn infer_literal(&mut self, lit: &Lit<'db>) -> InferTy<'db> {
        match lit.kind {
            LitKind::Bool => InferTy::Bool,
            LitKind::Integer => self.fresh_ty_var(), // Will be resolved by context or default to i32
            LitKind::Float => self.fresh_ty_var(), // Will be resolved by context or default to f64
            LitKind::Str => InferTy::Str,
        }
    }

    /// Infer the type of a path (variable reference).
    fn infer_path(&mut self, path: &Path<'db>, span: Span<'db>) -> InferTy<'db> {
        // For now, handle simple single-segment paths (variable names)
        if let Some(segment) = path.single_segment() {
            let name = segment.ident.name;

            // Check if it's a type parameter
            if self.is_type_param(name) {
                return InferTy::Param(name);
            }

            // Check if it's a variable in scope
            if let Some(ty) = self.lookup_var(name) {
                return ty;
            }

            // Check if it's a function (for function references)
            if self.lookup_function(name).is_some() {
                // Return a fresh type var for now - function type will be
                // determined at call site
                return self.fresh_ty_var();
            }

            self.emit_undefined_variable(name.text(self.db()), span);
            InferTy::Error
        } else if path.segments.len() == 2 {
            let enum_name = path.segments[0].ident.name;
            let variant_name = path.segments[1].ident.name;

            if let Some(enum_def) = self.lookup_enum(enum_name).cloned()
                && let Some((_, _variant_def)) = enum_def
                    .variants
                    .iter()
                    .find(|(name, _)| *name == variant_name)
            {
                return InferTy::Adt(enum_name);
            }

            // Could be a module::item path — return fresh var for now
            self.fresh_ty_var()
        } else {
            self.fresh_ty_var()
        }
    }

    /// Infer the type of a binary operation.
    fn infer_binary_op(
        &mut self,
        op: &BinOp<'db>,
        lhs: &Expr<'db>,
        rhs: &Expr<'db>,
        span: Span<'db>,
    ) -> InferTy<'db> {
        let lhs_ty = self.infer_expr(lhs);
        let rhs_ty = self.infer_expr(rhs);

        match op.node {
            // Arithmetic operators: operands must match, result is same type
            BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul | BinOpKind::Div | BinOpKind::Rem => {
                // Constrain operands to be the same type
                self.constrain_eq_with_kind(lhs_ty.clone(), rhs_ty, span, ConstraintKind::BinaryOp);
                lhs_ty
            }

            // Comparison operators: T -> T -> bool
            BinOpKind::Eq
            | BinOpKind::Ne
            | BinOpKind::Lt
            | BinOpKind::Le
            | BinOpKind::Gt
            | BinOpKind::Ge => {
                self.constrain_eq_with_kind(lhs_ty, rhs_ty, span, ConstraintKind::BinaryOp);
                InferTy::Bool
            }

            // Logical operators: bool -> bool -> bool
            BinOpKind::And | BinOpKind::Or => {
                self.constrain_eq_with_kind(
                    lhs_ty,
                    InferTy::Bool,
                    lhs.span,
                    ConstraintKind::BinaryOp,
                );
                self.constrain_eq_with_kind(
                    rhs_ty,
                    InferTy::Bool,
                    rhs.span,
                    ConstraintKind::BinaryOp,
                );
                InferTy::Bool
            }

            // Bitwise operators: operands must match, result is same type
            BinOpKind::BitAnd
            | BinOpKind::BitOr
            | BinOpKind::BitXor
            | BinOpKind::Shl
            | BinOpKind::Shr => {
                self.constrain_eq_with_kind(lhs_ty.clone(), rhs_ty, span, ConstraintKind::BinaryOp);
                lhs_ty
            }
        }
    }

    /// Infer the type of a unary operation.
    fn infer_unary_op(&mut self, op: UnOp, inner: &Expr<'db>, _span: Span<'db>) -> InferTy<'db> {
        let inner_ty = self.infer_expr(inner);
        match op {
            UnOp::Deref => {
                // Dereference: &T -> T, &mut T -> T, or *T -> T
                match &inner_ty {
                    InferTy::Ref(pointee, _) => (**pointee).clone(),
                    InferTy::Ptr(pointee) => (**pointee).clone(),
                    InferTy::Var(_) => {
                        // If the inner type is unknown, create a fresh var for the result
                        // and constrain the inner to be a pointer to that result
                        let result = self.fresh_ty_var();
                        let expected_ptr = InferTy::Ptr(Box::new(result.clone()));
                        self.constrain_eq(inner_ty, expected_ptr, inner.span);
                        result
                    }
                    InferTy::Error => InferTy::Error,
                    _ => {
                        self.emit_type_mismatch(
                            "&_ or *_",
                            &self.ty_to_string(&inner_ty),
                            inner.span,
                        );
                        InferTy::Error
                    }
                }
            }
            UnOp::Neg => inner_ty, // Negation preserves type
            UnOp::Not => inner_ty, // Logical NOT preserves type
        }
    }

    /// Infer the type of an address-of expression (`&expr` or `&mut expr`).
    fn infer_addr_of(
        &mut self,
        mutability: Mutability,
        inner: &Expr<'db>,
        span: Span<'db>,
    ) -> InferTy<'db> {
        let inner_ty = self.infer_expr(inner);

        // Check borrow rules on the inner expression
        if let ExprKind::Path(path) = &inner.kind
            && let Some(segment) = path.single_segment()
        {
            let name = segment.ident.name;
            if mutability.is_mut()
                && let Some(var_mut) = self.lookup_var_mutability(name)
                && var_mut.is_not()
            {
                self.emit_cannot_borrow_as_mutable(name.text(self.db()), inner.span);
            }
            self.record_borrow(name, mutability, span);
        }

        // Auto-deref through *T: `&x` where `x: *T` produces `&T`, not `&(*T)`
        let resolved = self.resolve(&inner_ty);
        match resolved {
            InferTy::Ptr(pointee) => InferTy::Ref(pointee, mutability),
            _ => InferTy::Ref(Box::new(inner_ty), mutability),
        }
    }

    /// Infer the type of a function call.
    fn infer_call(
        &mut self,
        callee: &Expr<'db>,
        args: &thin_vec::ThinVec<Box<Expr<'db>>>,
        span: Span<'db>,
    ) -> InferTy<'db> {
        // Try to get function name from callee
        if let ExprKind::Path(path) = &callee.kind {
            if let Some(segment) = path.single_segment() {
                let name = segment.ident.name;

                // Built-in: box(value) -> *T
                if name.text(self.db()) == "box" {
                    if args.len() != 1 {
                        self.emit_arity_mismatch(1, args.len(), span);
                        return InferTy::Error;
                    }
                    let arg_ty = self.infer_expr(&args[0]);
                    return InferTy::Ptr(Box::new(arg_ty));
                }

                if let Some(sig) = self.lookup_function(name).cloned() {
                    // Check argument count
                    if args.len() != sig.params.len() {
                        self.emit_arity_mismatch(sig.params.len(), args.len(), span);
                        return InferTy::Error;
                    }

                    // Instantiate generic parameters with fresh type variables
                    let mut subst: HashMap<Symbol<'db>, InferTy<'db>> = HashMap::new();
                    for type_param in &sig.type_params {
                        subst.insert(*type_param, self.fresh_ty_var());
                    }

                    // Check each argument
                    for (arg, (_, param_ty)) in args.iter().zip(sig.params.iter()) {
                        let arg_ty = self.infer_expr(arg);
                        let expected_ty = self.substitute(param_ty, &subst);
                        self.constrain_eq_with_kind(
                            arg_ty,
                            expected_ty,
                            arg.span,
                            ConstraintKind::FunctionArg,
                        );
                    }

                    // Return the instantiated return type
                    return self.substitute(&sig.return_ty, &subst);
                }

                self.emit_undefined_function(name.text(self.db()), span);
                return InferTy::Error;
            }

            if path.segments.len() == 2 {
                let enum_name = path.segments[0].ident.name;
                let variant_name = path.segments[1].ident.name;

                if let Some(enum_def) = self.lookup_enum(enum_name).cloned()
                    && let Some((_, variant_def)) = enum_def
                        .variants
                        .iter()
                        .find(|(name, _)| *name == variant_name)
                    && let crate::context::EnumVariantDef::Tuple(field_tys) = variant_def
                {
                    if args.len() != field_tys.len() {
                        self.emit_arity_mismatch(field_tys.len(), args.len(), span);
                        return InferTy::Error;
                    }
                    for (arg, expected_ty) in args.iter().zip(field_tys.iter()) {
                        let arg_ty = self.infer_expr(arg);
                        self.constrain_eq_with_kind(
                            arg_ty,
                            expected_ty.clone(),
                            arg.span,
                            ConstraintKind::FunctionArg,
                        );
                    }
                    return InferTy::Adt(enum_name);
                }
            }
        }

        // Indirect call - infer callee type
        let callee_ty = self.infer_expr(callee);
        let arg_tys: Vec<_> = args.iter().map(|a| self.infer_expr(a)).collect();
        let ret_ty = self.fresh_ty_var();

        self.constrain_eq(
            callee_ty,
            InferTy::Fn(arg_tys, Box::new(ret_ty.clone())),
            span,
        );

        ret_ty
    }

    /// Infer the type of an if expression.
    fn infer_if(
        &mut self,
        cond: &Expr<'db>,
        then_block: &Block<'db>,
        else_expr: Option<&Expr<'db>>,
        span: Span<'db>,
    ) -> InferTy<'db> {
        // Condition must be bool
        let cond_ty = self.infer_expr(cond);
        self.constrain_eq_with_kind(
            cond_ty,
            InferTy::Bool,
            cond.span,
            ConstraintKind::IfCondition,
        );

        // Infer then branch
        let then_ty = self.infer_block(then_block);

        // Infer else branch (if present)
        if let Some(else_expr) = else_expr {
            let else_ty = self.infer_expr(else_expr);
            // Both branches must have same type
            self.constrain_eq_with_kind(then_ty.clone(), else_ty, span, ConstraintKind::IfBranches);
            then_ty
        } else {
            // No else branch - if expression has unit type
            then_ty
        }
    }

    /// Infer the type of a block.
    pub fn infer_block(&mut self, block: &Block<'db>) -> InferTy<'db> {
        self.push_scope();

        let mut result_ty = InferTy::unit(); // Default to unit

        for stmt in &block.stmts {
            result_ty = self.infer_stmt(stmt);
        }

        self.pop_scope();
        result_ty
    }

    /// Infer the type of a statement.
    fn infer_stmt(&mut self, stmt: &Stmt<'db>) -> InferTy<'db> {
        match &stmt.kind {
            StmtKind::Let(local) => {
                self.infer_local(local);
                InferTy::unit()
            }
            StmtKind::Expr(expr) => self.infer_expr(expr),
            StmtKind::Semi(expr) => {
                let ty = self.infer_expr(expr);
                if ty.is_never() {
                    InferTy::Never
                } else {
                    InferTy::unit()
                }
            }
            StmtKind::Item(_) => {
                // Items are handled during signature collection
                InferTy::unit()
            }
            StmtKind::Empty => InferTy::unit(),
        }
    }

    /// Infer types for a local variable declaration.
    fn infer_local(&mut self, local: &Local<'db>) {
        // Get the declared type (if any)
        let declared_ty = local.ty.as_ref().map(|t| self.lower_ast_ty(t));

        // Get the initializer type (if any)
        let init_ty = match &local.kind {
            LocalKind::Init(expr) => Some(self.infer_expr(expr)),
            LocalKind::Decl => None,
        };

        // Determine the variable's type
        let var_ty = match (declared_ty, init_ty) {
            (Some(decl), Some(init)) => {
                // Both declared and initialized - they must match
                self.constrain_eq_with_kind(
                    decl.clone(),
                    init,
                    local.span,
                    ConstraintKind::LetBinding,
                );
                decl
            }
            (Some(decl), None) => decl,
            (None, Some(init)) => init,
            (None, None) => self.fresh_ty_var(),
        };

        // Record the local variable's type
        self.record_local_type(local.id, var_ty.clone());

        // Bind the variable with its mutability
        if let PatKind::Ident(binding_mode, ident, _) = &local.pat.kind {
            self.define_var_with_mutability(ident.name, var_ty, binding_mode.1);
        }
    }

    /// Infer the type of a return expression.
    fn infer_return(&mut self, value: Option<&Expr<'db>>, span: Span<'db>) -> InferTy<'db> {
        let return_ty = match value {
            Some(expr) => self.infer_expr(expr),
            None => InferTy::unit(),
        };

        // Constrain against function's declared return type
        if let Some(expected) = self.return_ty().cloned() {
            self.constrain_eq_with_kind(return_ty, expected, span, ConstraintKind::FunctionReturn);
        }

        InferTy::Never // Return never returns (diverges)
    }

    /// Infer the type of an assignment.
    fn infer_assign(&mut self, lhs: &Expr<'db>, rhs: &Expr<'db>, span: Span<'db>) -> InferTy<'db> {
        let lhs_ty = self.infer_expr(lhs);
        self.check_assign_mutability(lhs);
        let rhs_ty = self.infer_expr(rhs);

        self.constrain_eq_with_kind(lhs_ty, rhs_ty, span, ConstraintKind::Assignment);

        InferTy::unit() // Assignment returns unit
    }

    /// Infer the type of a compound assignment.
    fn infer_assign_op(
        &mut self,
        _op: &AssignOp<'db>,
        lhs: &Expr<'db>,
        rhs: &Expr<'db>,
        span: Span<'db>,
    ) -> InferTy<'db> {
        let lhs_ty = self.infer_expr(lhs);
        self.check_assign_mutability(lhs);
        let rhs_ty = self.infer_expr(rhs);

        // Compound assignment requires operands to have matching types
        self.constrain_eq_with_kind(lhs_ty, rhs_ty, span, ConstraintKind::BinaryOp);

        InferTy::unit() // Assignment returns unit
    }

    /// Check that the LHS of an assignment is mutable.
    /// Emits an error if assigning to an immutable variable.
    fn check_assign_mutability(&self, lhs: &Expr<'db>) {
        match &lhs.kind {
            ExprKind::Path(path) => {
                if let Some(segment) = path.single_segment()
                    && let Some(mutability) = self.lookup_var_mutability(segment.ident.name)
                    && mutability.is_not()
                {
                    self.emit_immutable_assign_error(segment.ident.name.text(self.db()), lhs.span);
                }
            }
            ExprKind::Field(base, _) => {
                // For `p.x = 5`, check the root variable's mutability
                self.check_assign_mutability(base);
            }
            ExprKind::Unary(UnOp::Deref, inner) => {
                let inner_ty = self.resolve(&self.lookup_expr_type_infer(inner.id));
                match &inner_ty {
                    InferTy::Ref(_, Mutability::Not) => {
                        // Can't write through &T
                        self.emit_immutable_ref_deref_error(lhs.span);
                    }
                    InferTy::Ref(_, Mutability::Mut) => {
                        // &mut T — writing is allowed regardless of binding mutability
                    }
                    _ => {
                        // *T (GC pointer) — check the variable's binding mutability
                        self.check_assign_mutability(inner);
                    }
                }
            }
            _ => {}
        }
    }

    /// Infer the type of an array literal.
    fn infer_array(
        &mut self,
        elements: &thin_vec::ThinVec<Box<Expr<'db>>>,
        _span: Span<'db>,
    ) -> InferTy<'db> {
        if elements.is_empty() {
            // Empty array - element type is unknown
            let elem_ty = self.fresh_ty_var();
            return InferTy::App(Symbol::new(self.db(), "Array".to_string()), vec![elem_ty]);
        }

        // All elements must have same type
        let first_ty = self.infer_expr(&elements[0]);
        for elem in elements.iter().skip(1) {
            let elem_ty = self.infer_expr(elem);
            self.constrain_eq_with_kind(
                first_ty.clone(),
                elem_ty,
                elem.span,
                ConstraintKind::ArrayElement,
            );
        }

        InferTy::App(Symbol::new(self.db(), "Array".to_string()), vec![first_ty])
    }

    /// Convert an AST type to an InferTy.
    /// Infer the type of a struct initialization expression.
    fn infer_struct_init(
        &mut self,
        path: &Path<'db>,
        fields: &thin_vec::ThinVec<scrap_ast::expr::ExprField<'db>>,
        span: Span<'db>,
    ) -> InferTy<'db> {
        // Check for enum struct variant: `Message::Move { x: 1, y: 2 }`
        if path.segments.len() == 2 {
            let enum_name = path.segments[0].ident.name;
            let variant_name = path.segments[1].ident.name;

            if let Some(enum_def) = self.lookup_enum(enum_name).cloned()
                && let Some((_, variant_def)) = enum_def
                    .variants
                    .iter()
                    .find(|(name, _)| *name == variant_name)
                && let crate::context::EnumVariantDef::Struct(field_defs) = variant_def
            {
                for field_init in fields.iter() {
                    let field_ty = self.infer_expr(&field_init.expr);
                    if let Some((_, expected_ty)) = field_defs
                        .iter()
                        .find(|(name, _)| *name == field_init.ident.name)
                    {
                        self.constrain_eq(field_ty, expected_ty.clone(), field_init.span);
                    }
                }
                return InferTy::Adt(enum_name);
            }
        }

        let struct_name = match path.single_segment() {
            Some(seg) => seg.ident.name,
            None => {
                self.emit_type_mismatch("struct name", "multi-segment path", span);
                return InferTy::Error;
            }
        };

        let struct_def = match self.lookup_struct(struct_name) {
            Some(def) => def.clone(),
            None => {
                self.emit_undefined_variable(struct_name.text(self.db()), span);
                return InferTy::Error;
            }
        };

        // Check for unknown fields (provided but not in struct def)
        let mut has_error = false;
        for field_init in fields.iter() {
            let field_exists = struct_def
                .fields
                .iter()
                .any(|(name, _)| *name == field_init.ident.name);
            if !field_exists {
                let sn = struct_name.text(self.db());
                let fn_ = field_init.ident.name.text(self.db());
                let note = if struct_def.fields.is_empty() {
                    "all struct fields are already assigned".to_string()
                } else {
                    let available: Vec<_> = struct_def
                        .fields
                        .iter()
                        .map(|(n, _)| format!("`{}`", n.text(self.db())))
                        .collect();
                    format!("available fields are: {}", available.join(", "))
                };
                self.emit_unknown_struct_field(sn, fn_, field_init.ident.span, note);
                has_error = true;
            }
        }

        // Check for missing fields (in struct def but not provided)
        for (def_name, _) in struct_def.fields.iter() {
            let provided = fields.iter().any(|f| f.ident.name == *def_name);
            if !provided {
                let sn = struct_name.text(self.db());
                let fn_ = def_name.text(self.db());
                self.emit_missing_struct_field(sn, fn_, span);
                has_error = true;
            }
        }

        if has_error {
            return InferTy::Error;
        }

        // Constrain field types
        for field_init in fields.iter() {
            let field_ty = self.infer_expr(&field_init.expr);
            if let Some((_, expected_ty)) = struct_def
                .fields
                .iter()
                .find(|(name, _)| *name == field_init.ident.name)
            {
                self.constrain_eq(field_ty, expected_ty.clone(), field_init.span);
            }
        }

        InferTy::Adt(struct_name)
    }

    /// Infer the type of a field access expression.
    fn infer_field_access(
        &mut self,
        base: &Expr<'db>,
        field_ident: &scrap_shared::ident::Ident<'db>,
        span: Span<'db>,
    ) -> InferTy<'db> {
        let base_ty = self.infer_expr(base);
        let resolved_base = self.resolve(&base_ty);

        match &resolved_base {
            InferTy::Adt(struct_name) => {
                let struct_def = match self.lookup_struct(*struct_name) {
                    Some(def) => def.clone(),
                    None => {
                        self.emit_type_mismatch("struct", &self.ty_to_string(&base_ty), span);
                        return InferTy::Error;
                    }
                };

                let field_name = field_ident.name;
                if let Some((_, field_ty)) = struct_def
                    .fields
                    .iter()
                    .find(|(name, _)| *name == field_name)
                {
                    field_ty.clone()
                } else {
                    self.emit_undefined_variable(
                        &format!(
                            "{}.{}",
                            struct_name.text(self.db()),
                            field_name.text(self.db())
                        ),
                        span,
                    );
                    InferTy::Error
                }
            }
            InferTy::Error => InferTy::Error,
            _ => {
                self.emit_type_mismatch("struct type", &self.ty_to_string(&base_ty), span);
                InferTy::Error
            }
        }
    }

    pub fn lower_ast_ty(&mut self, ty: &Ty<'db>) -> InferTy<'db> {
        match &ty.kind {
            TyKind::Path(path) => {
                if let Some(segment) = path.single_segment() {
                    let name_str = segment.ident.name.text(self.db());
                    match name_str.as_str() {
                        // Signed integers
                        "i8" => InferTy::Int(IntTy::I8),
                        "i16" => InferTy::Int(IntTy::I16),
                        "i32" => InferTy::Int(IntTy::I32),
                        "i64" => InferTy::Int(IntTy::I64),
                        "i128" => InferTy::Int(IntTy::I128),
                        "isize" => InferTy::Int(IntTy::Isize),
                        // Unsigned integers
                        "u8" => InferTy::Uint(UintTy::U8),
                        "u16" => InferTy::Uint(UintTy::U16),
                        "u32" => InferTy::Uint(UintTy::U32),
                        "u64" => InferTy::Uint(UintTy::U64),
                        "u128" => InferTy::Uint(UintTy::U128),
                        "usize" => InferTy::Uint(UintTy::Usize),
                        // Floats
                        "f16" => InferTy::Float(FloatTy::F16),
                        "f32" => InferTy::Float(FloatTy::F32),
                        "f64" => InferTy::Float(FloatTy::F64),
                        "f128" => InferTy::Float(FloatTy::F128),
                        // Legacy alias
                        "int" => InferTy::Int(IntTy::I32),
                        // Other primitives
                        "void" => InferTy::Void,
                        "bool" => InferTy::Bool,
                        "String" => InferTy::Str,
                        _ => {
                            // Check if it's a type parameter
                            let sym = segment.ident.name;
                            if self.is_type_param(sym) {
                                InferTy::Param(sym)
                            } else {
                                InferTy::Adt(sym)
                            }
                        }
                    }
                } else {
                    InferTy::Error
                }
            }
            TyKind::Tup(elems) => {
                let elem_tys: Vec<_> = elems.iter().map(|e| self.lower_ast_ty(e)).collect();
                InferTy::Tuple(elem_tys)
            }
            TyKind::Ref(inner, mutability) => {
                let inner_ty = self.lower_ast_ty(inner);
                InferTy::Ref(Box::new(inner_ty), *mutability)
            }
            TyKind::Ptr(inner) => {
                let inner_ty = self.lower_ast_ty(inner);
                InferTy::Ptr(Box::new(inner_ty))
            }
            TyKind::Never => InferTy::Never,
            TyKind::Dummy => self.fresh_ty_var(),
            TyKind::Err(_) => InferTy::Error,
        }
    }

    /// Substitute type parameters with their instantiated types.
    pub fn substitute(
        &self,
        ty: &InferTy<'db>,
        subst: &HashMap<Symbol<'db>, InferTy<'db>>,
    ) -> InferTy<'db> {
        match ty {
            InferTy::Param(name) => subst.get(name).cloned().unwrap_or_else(|| ty.clone()),
            InferTy::App(name, args) => {
                let new_args: Vec<_> = args.iter().map(|a| self.substitute(a, subst)).collect();
                InferTy::App(*name, new_args)
            }
            InferTy::Fn(params, ret) => {
                let new_params: Vec<_> = params.iter().map(|p| self.substitute(p, subst)).collect();
                let new_ret = self.substitute(ret, subst);
                InferTy::Fn(new_params, Box::new(new_ret))
            }
            InferTy::Tuple(elems) => {
                let new_elems: Vec<_> = elems.iter().map(|e| self.substitute(e, subst)).collect();
                InferTy::Tuple(new_elems)
            }
            InferTy::Ref(inner, m) => InferTy::Ref(Box::new(self.substitute(inner, subst)), *m),
            InferTy::Ptr(inner) => InferTy::Ptr(Box::new(self.substitute(inner, subst))),
            _ => ty.clone(),
        }
    }

    /// Infer the type of a match expression.
    fn infer_match(
        &mut self,
        scrutinee: &Expr<'db>,
        arms: &[scrap_ast::expr::Arm<'db>],
        span: Span<'db>,
    ) -> InferTy<'db> {
        // Infer scrutinee type
        let scrutinee_ty = self.infer_expr(scrutinee);

        // Infer each arm body; all arms must have the same type
        let result_ty = self.fresh_ty_var();
        for arm in arms {
            self.push_scope();
            self.bind_match_pattern(&arm.pat, &scrutinee_ty);
            let arm_ty = self.infer_expr(&arm.body);
            self.constrain_eq_with_kind(result_ty.clone(), arm_ty, span, ConstraintKind::MatchArm);
            self.pop_scope();
        }

        result_ty
    }

    /// Infer the type of a method call expression.
    fn infer_method_call(
        &mut self,
        receiver: &Expr<'db>,
        method: &scrap_shared::ident::Ident<'db>,
        args: &thin_vec::ThinVec<Box<Expr<'db>>>,
        span: Span<'db>,
    ) -> InferTy<'db> {
        let recv_ty = self.infer_expr(receiver);
        let resolved = self.resolve(&recv_ty);

        let type_name = match &resolved {
            InferTy::Adt(name) => *name,
            _ => {
                self.emit_type_mismatch("struct or enum", &self.ty_to_string(&resolved), span);
                return InferTy::Error;
            }
        };

        // Construct mangled name: TypeName::method_name
        let mangled = Symbol::new(
            self.db(),
            format!(
                "{}::{}",
                type_name.text(self.db()),
                method.name.text(self.db())
            ),
        );

        let sig = match self.lookup_function(mangled) {
            Some(sig) => sig.clone(),
            None => {
                let msg = format!(
                    "{}::{}",
                    type_name.text(self.db()),
                    method.name.text(self.db())
                );
                self.emit_undefined_function(&msg, span);
                return InferTy::Error;
            }
        };

        // Check arg count (exclude self parameter)
        let method_params = &sig.params[1..];
        if args.len() != method_params.len() {
            self.emit_arity_mismatch(method_params.len(), args.len(), span);
            return InferTy::Error;
        }

        // Constrain receiver type
        self.constrain_eq(recv_ty, sig.params[0].1.clone(), span);

        // Constrain argument types
        for (arg, (_, param_ty)) in args.iter().zip(method_params) {
            let arg_ty = self.infer_expr(arg);
            self.constrain_eq_with_kind(
                arg_ty,
                param_ty.clone(),
                span,
                ConstraintKind::FunctionArg,
            );
        }

        sig.return_ty.clone()
    }

    /// Bind variables from a match pattern into the current scope.
    fn bind_match_pattern(&mut self, pat: &scrap_ast::pat::Pat<'db>, scrutinee_ty: &InferTy<'db>) {
        use scrap_ast::pat::PatKind;
        match &pat.kind {
            PatKind::Wildcard | PatKind::Missing | PatKind::Lit(_) => {}
            PatKind::Ident(_, ident, _) => {
                // Simple binding: `val` captures the whole matched value
                self.define_var(ident.name, scrutinee_ty.clone());
            }
            PatKind::Path(_) => {
                // Unit variant pattern (e.g. Option::None) - no bindings
            }
            PatKind::TupleStruct(path, sub_pats) => {
                if path.segments.len() == 2 {
                    let enum_name = path.segments[0].ident.name;
                    let variant_name = path.segments[1].ident.name;
                    if let Some(enum_def) = self.lookup_enum(enum_name).cloned()
                        && let Some((_, crate::context::EnumVariantDef::Tuple(field_tys))) =
                            enum_def
                                .variants
                                .iter()
                                .find(|(name, _)| *name == variant_name)
                    {
                        for (sub_pat, field_ty) in sub_pats.iter().zip(field_tys.iter()) {
                            self.bind_match_pattern(sub_pat, field_ty);
                        }
                    }
                }
            }
            PatKind::Struct(path, field_pats) => {
                if path.segments.len() == 2 {
                    let enum_name = path.segments[0].ident.name;
                    let variant_name = path.segments[1].ident.name;
                    if let Some(enum_def) = self.lookup_enum(enum_name).cloned()
                        && let Some((_, crate::context::EnumVariantDef::Struct(field_defs))) =
                            enum_def
                                .variants
                                .iter()
                                .find(|(name, _)| *name == variant_name)
                    {
                        for fp in field_pats {
                            if let Some((_, field_ty)) =
                                field_defs.iter().find(|(name, _)| *name == fp.ident.name)
                            {
                                self.bind_match_pattern(&fp.pat, field_ty);
                            }
                        }
                    }
                }
            }
        }
    }
}
