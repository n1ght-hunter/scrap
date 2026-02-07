//! Type inference for expressions.

use std::collections::HashMap;

use scrap_ast::{
    block::Block,
    expr::{Expr, ExprKind},
    lit::{Lit, LitKind},
    local::{Local, LocalKind},
    operators::{AssignOp, BinOp, BinOpKind},
    pat::PatKind,
    stmt::{Stmt, StmtKind},
    typedef::{Ty, TyKind},
};
use scrap_shared::types::{FloatTy, IntTy, UintTy};
use scrap_shared::{ident::Symbol, path::Path};
use scrap_span::Span;

use crate::{
    constraints::ConstraintKind,
    context::TypeContext,
    types::InferTy,
};

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
            LitKind::Float => self.fresh_ty_var(),   // Will be resolved by context or default to f64
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

            self.emit_undefined_variable(&name.text(self.db()), span);
            InferTy::Error
        } else {
            // Multi-segment paths (module::item) - handle later
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
            BinOpKind::Eq | BinOpKind::Ne | BinOpKind::Lt | BinOpKind::Le | BinOpKind::Gt | BinOpKind::Ge => {
                self.constrain_eq_with_kind(lhs_ty, rhs_ty, span, ConstraintKind::BinaryOp);
                InferTy::Bool
            }

            // Logical operators: bool -> bool -> bool
            BinOpKind::And | BinOpKind::Or => {
                self.constrain_eq_with_kind(lhs_ty, InferTy::Bool, lhs.span, ConstraintKind::BinaryOp);
                self.constrain_eq_with_kind(rhs_ty, InferTy::Bool, rhs.span, ConstraintKind::BinaryOp);
                InferTy::Bool
            }

            // Bitwise operators: operands must match, result is same type
            BinOpKind::BitAnd | BinOpKind::BitOr | BinOpKind::BitXor | BinOpKind::Shl | BinOpKind::Shr => {
                self.constrain_eq_with_kind(lhs_ty.clone(), rhs_ty, span, ConstraintKind::BinaryOp);
                lhs_ty
            }
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
                        self.constrain_eq_with_kind(arg_ty, expected_ty, arg.span, ConstraintKind::FunctionArg);
                    }

                    // Return the instantiated return type
                    return self.substitute(&sig.return_ty, &subst);
                }

                self.emit_undefined_function(&name.text(self.db()), span);
                return InferTy::Error;
            }
        }

        // Indirect call - infer callee type
        let callee_ty = self.infer_expr(callee);
        let arg_tys: Vec<_> = args.iter().map(|a| self.infer_expr(a)).collect();
        let ret_ty = self.fresh_ty_var();

        self.constrain_eq(callee_ty, InferTy::Fn(arg_tys, Box::new(ret_ty.clone())), span);

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
        self.constrain_eq_with_kind(cond_ty, InferTy::Bool, cond.span, ConstraintKind::IfCondition);

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
                if ty.is_never() { InferTy::Never } else { InferTy::unit() }
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
                self.constrain_eq_with_kind(decl.clone(), init, local.span, ConstraintKind::LetBinding);
                decl
            }
            (Some(decl), None) => decl,
            (None, Some(init)) => init,
            (None, None) => self.fresh_ty_var(),
        };

        // Record the local variable's type
        self.record_local_type(local.id, var_ty.clone());

        // Bind the variable
        if let PatKind::Ident(_, ident, _) = &local.pat.kind {
            self.define_var(ident.name, var_ty);
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
    fn infer_assign(
        &mut self,
        lhs: &Expr<'db>,
        rhs: &Expr<'db>,
        span: Span<'db>,
    ) -> InferTy<'db> {
        let lhs_ty = self.infer_expr(lhs);
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
        let rhs_ty = self.infer_expr(rhs);

        // Compound assignment requires operands to have matching types
        self.constrain_eq_with_kind(lhs_ty, rhs_ty, span, ConstraintKind::BinaryOp);

        InferTy::unit() // Assignment returns unit
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
            self.constrain_eq_with_kind(first_ty.clone(), elem_ty, elem.span, ConstraintKind::ArrayElement);
        }

        InferTy::App(Symbol::new(self.db(), "Array".to_string()), vec![first_ty])
    }

    /// Convert an AST type to an InferTy.
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
                let new_params: Vec<_> =
                    params.iter().map(|p| self.substitute(p, subst)).collect();
                let new_ret = self.substitute(ret, subst);
                InferTy::Fn(new_params, Box::new(new_ret))
            }
            InferTy::Tuple(elems) => {
                let new_elems: Vec<_> =
                    elems.iter().map(|e| self.substitute(e, subst)).collect();
                InferTy::Tuple(new_elems)
            }
            _ => ty.clone(),
        }
    }
}
