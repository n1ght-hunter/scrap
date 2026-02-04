//! Expression lowering from AST to IR
//!
//! This module handles the conversion of AST expressions to IR operands and rvalues.

mod array;
mod assign;
mod binary;
mod block;
mod call;
mod control;
mod lit;
mod path;

use scrap_ast::expr::{Expr, ExprKind};
use scrap_ir as ir;

use crate::{lowerer::ExprLowerer, BuilderError, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower an expression to an operand
    pub fn lower_expr(&mut self, expr: &Expr<'db>) -> MResult<ir::Operand<'db>> {
        match &expr.kind {
            ExprKind::Lit(lit) => self.lower_literal(lit),
            ExprKind::Path(path) => self.lower_path(path),
            ExprKind::Binary(op, lhs, rhs) => self.lower_binary_op(op, lhs, rhs),
            ExprKind::Paren(inner) => self.lower_expr(inner),
            ExprKind::Assign(lhs, rhs, _span) => self.lower_assign(lhs, rhs),
            ExprKind::AssignOp(op, lhs, rhs) => self.lower_assign_op(op, lhs, rhs),
            ExprKind::If(cond, then_block, else_expr) => {
                self.lower_if_expr(cond, then_block, else_expr.as_deref())
            }
            ExprKind::Return(value) => self.lower_return(value.as_deref()),
            ExprKind::Block(block) => self.lower_block_expr(block),
            ExprKind::Array(elements) => self.lower_array(elements),
            ExprKind::Call(func, args) => self.lower_call(func, args),
            ExprKind::Err => Err(BuilderError::LowerExpressionError),
            _ => {
                // Unsupported expression types for now
                Err(BuilderError::LowerExpressionError)
            }
        }
    }
}
