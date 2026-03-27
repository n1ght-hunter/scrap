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
mod match_expr;
mod method_call;
mod path;
mod struct_expr;
mod unary;

use scrap_ast::expr::{Expr, ExprKind};
use scrap_ast::operators::UnOp;
use scrap_ir as ir;

use crate::{BuilderError, MResult, lowerer::ExprLowerer};

impl<'db> ExprLowerer<'db> {
    /// Lower an expression to an operand
    pub fn lower_expr(&mut self, expr: &Expr<'db>) -> MResult<ir::Operand<'db>> {
        match &expr.kind {
            ExprKind::Lit(lit) => self.lower_literal(lit, expr.id),
            ExprKind::Path(path) => self.lower_path(path),
            ExprKind::Binary(_, _, _) => self.lower_binary_op(expr),
            ExprKind::Paren(inner) => self.lower_expr(inner),
            ExprKind::Assign(lhs, rhs, _span) => self.lower_assign(lhs, rhs),
            ExprKind::AssignOp(op, lhs, rhs) => self.lower_assign_op(op, lhs, rhs),
            ExprKind::If(cond, then_block, else_expr) => {
                self.lower_if_expr(cond, then_block, else_expr.as_deref(), expr.id)
            }
            ExprKind::Return(value) => self.lower_return(value.as_deref()),
            ExprKind::Block(block) => self.lower_block_expr(block),
            ExprKind::Array(_) => self.lower_array(expr),
            ExprKind::Call(_, _) => self.lower_call(expr),
            ExprKind::Unary(UnOp::Deref, inner) => self.lower_deref(inner, expr.id),
            ExprKind::Unary(UnOp::Neg, inner) => self.lower_unary_neg(inner, expr.id),
            ExprKind::Unary(UnOp::Not, inner) => self.lower_unary_not(inner, expr.id),
            ExprKind::Struct(struct_expr) => self.lower_struct_init(struct_expr, expr.id),
            ExprKind::Field(base, field_ident) => {
                self.lower_field_access(base, field_ident, expr.id)
            }
            ExprKind::Match(scrutinee, arms) => self.lower_match(scrutinee, arms),
            ExprKind::MethodCall(receiver, method, args) => {
                self.lower_method_call(receiver, method, args, expr.id)
            }
            ExprKind::AddrOf(mutability, inner) => self.lower_addr_of(*mutability, inner, expr.id),
            ExprKind::Spawn(inner) => self.lower_spawn(inner),
            ExprKind::Loop(block) => self.lower_loop_expr(block),
            ExprKind::While(cond, block) => self.lower_while_expr(cond, block),
            ExprKind::Break(_) => self.lower_break(),
            ExprKind::Continue => self.lower_continue(),
            ExprKind::Err => Err(BuilderError::LowerExpressionError),
        }
    }

    /// Lower an expression directly into a destination place.
    /// Avoids allocating a temporary — the result is written to `dest`.
    pub fn lower_expr_into(&mut self, expr: &Expr<'db>, dest: ir::Place<'db>) -> MResult<()> {
        match &expr.kind {
            ExprKind::Lit(lit) => self.lower_literal_into(lit, expr.id, dest),
            ExprKind::Binary(_, _, _) => self.lower_binary_op_into(expr, dest),
            ExprKind::Call(_, _) => self.lower_call_into(expr, dest),
            ExprKind::Paren(inner) => self.lower_expr_into(inner, dest),
            ExprKind::Struct(struct_expr) => self.lower_struct_init_into(struct_expr, dest),
            // For other expressions, fall back to lower_expr + assign
            _ => {
                let operand = self.lower_expr(expr)?;
                self.emit_assign(dest, ir::Rvalue::Use(operand));
                Ok(())
            }
        }
    }
}
