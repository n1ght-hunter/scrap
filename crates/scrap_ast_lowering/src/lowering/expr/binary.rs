//! Binary operation lowering

use scrap_ast::{
    expr::Expr,
    operators::{BinOp, BinOpKind},
};
use scrap_ir as ir;

use crate::{lowerer::ExprLowerer, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower a binary operation to an operand
    pub(crate) fn lower_binary_op(
        &mut self,
        binary_expr: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        // Extract operator and operands from binary expression
        let (op, lhs, rhs) = match &binary_expr.kind {
            scrap_ast::expr::ExprKind::Binary(o, l, r) => (o, l, r),
            _ => return Err(crate::BuilderError::LowerExpressionError),
        };

        // Recursively lower the left and right operands
        let lhs_operand = self.lower_expr(lhs)?;
        let rhs_operand = self.lower_expr(rhs)?;

        // Convert AST binary operator to IR binary operator
        let ir_op = self.convert_bin_op(op.node)?;

        // Allocate a temporary for the result using type from type table
        let result_ty = self.lookup_and_convert_type(binary_expr.id);
        let temp = self.allocate_temp(result_ty);

        // Emit assignment: temp = lhs op rhs
        let place = ir::Place::Local(temp);
        let rvalue = ir::Rvalue::BinaryOp(ir_op, lhs_operand, rhs_operand);
        self.emit_assign(place, rvalue);

        // Return reference to the result temporary
        Ok(ir::Operand::Place(ir::Place::Local(temp)))
    }

    /// Convert AST binary operator to IR binary operator
    pub(crate) fn convert_bin_op(&self, op: BinOpKind) -> MResult<ir::BinOp> {
        match op {
            // Arithmetic operators
            BinOpKind::Add => Ok(ir::BinOp::Add),
            BinOpKind::Sub => Ok(ir::BinOp::Sub),
            BinOpKind::Mul => Ok(ir::BinOp::Mul),
            BinOpKind::Div => Ok(ir::BinOp::Div),
            BinOpKind::Rem => Ok(ir::BinOp::Rem),

            // Logical operators
            BinOpKind::And => Ok(ir::BinOp::And),
            BinOpKind::Or => Ok(ir::BinOp::Or),

            // Bitwise operators
            BinOpKind::BitXor => Ok(ir::BinOp::BitXor),
            BinOpKind::BitAnd => Ok(ir::BinOp::BitAnd),
            BinOpKind::BitOr => Ok(ir::BinOp::BitOr),
            BinOpKind::Shl => Ok(ir::BinOp::Shl),
            BinOpKind::Shr => Ok(ir::BinOp::Shr),

            // Comparison operators
            BinOpKind::Eq => Ok(ir::BinOp::Eq),
            BinOpKind::Lt => Ok(ir::BinOp::Lt),
            BinOpKind::Le => Ok(ir::BinOp::Le),
            BinOpKind::Ne => Ok(ir::BinOp::Ne),
            BinOpKind::Ge => Ok(ir::BinOp::Ge),
            BinOpKind::Gt => Ok(ir::BinOp::Gt),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[scrap_macros::salsa_test]
    fn test_lower_binary_add(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 5);
        let rhs = create_int_lit(db, 3);
        let expr = create_binary_expr(db, BinOpKind::Add, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        // Should have 3 locals: lhs temp, rhs temp, result temp
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_sub(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 10);
        let rhs = create_int_lit(db, 4);
        let expr = create_binary_expr(db, BinOpKind::Sub, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_mul(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 6);
        let rhs = create_int_lit(db, 7);
        let expr = create_binary_expr(db, BinOpKind::Mul, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_comparison(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 5);
        let rhs = create_int_lit(db, 10);
        let expr = create_binary_expr(db, BinOpKind::Lt, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_logical_and(db: &dyn scrap_shared::Db) {
        let lhs = create_bool_lit(db, true);
        let rhs = create_bool_lit(db, false);
        let expr = create_binary_expr(db, BinOpKind::And, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_bitwise(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 5);
        let rhs = create_int_lit(db, 3);
        let expr = create_binary_expr(db, BinOpKind::BitAnd, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_nested_binary(db: &dyn scrap_shared::Db) {
        // (5 + 3) * 2
        let five = create_int_lit(db, 5);
        let three = create_int_lit(db, 3);
        let add_expr = create_binary_expr(db, BinOpKind::Add, five, three);

        let two = create_int_lit(db, 2);
        let mul_expr = create_binary_expr(db, BinOpKind::Mul, add_expr, two);

        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));
        let result = lowerer.lower_expr(&mul_expr);
        assert!(result.is_ok());

        // Should have: 5_temp, 3_temp, add_result, 2_temp, mul_result = 5 locals
        assert_eq!(lowerer.local_decls.len(), 5);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_parenthesized(db: &dyn scrap_shared::Db) {
        let inner = create_int_lit(db, 42);
        let expr = create_paren_expr(db, inner);

        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        // Parentheses don't add anything, just unwrap
        assert_eq!(lowerer.local_decls.len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_operator_conversion(db: &dyn scrap_shared::Db) {
        let lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Test all arithmetic operators
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Add).unwrap(), ir::BinOp::Add);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Sub).unwrap(), ir::BinOp::Sub);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Mul).unwrap(), ir::BinOp::Mul);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Div).unwrap(), ir::BinOp::Div);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Rem).unwrap(), ir::BinOp::Rem);

        // Test comparison operators
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Eq).unwrap(), ir::BinOp::Eq);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Lt).unwrap(), ir::BinOp::Lt);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Gt).unwrap(), ir::BinOp::Gt);
    }
}
