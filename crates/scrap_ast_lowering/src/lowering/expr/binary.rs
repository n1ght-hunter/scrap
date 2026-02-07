//! Binary operation lowering

use scrap_ast::{
    expr::Expr,
    operators::BinOpKind,
};
use scrap_ir as ir;

use crate::{lowerer::ExprLowerer, MResult};

/// Whether an operation should use checked (overflow-detecting) semantics.
enum LoweredBinOp {
    /// Unchecked: produces a single value via `Rvalue::Intrinsic`
    Unchecked(ir::IntrinsicOp),
    /// Checked: produces `(T, bool)` via `Rvalue::Intrinsic`, then `Assert`
    Checked(ir::IntrinsicOp),
}

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

        // Look up the result type from the type table
        let result_ty = self.lookup_and_convert_type(binary_expr.id);

        // Determine whether this is a checked or unchecked operation
        let lowered = self.classify_bin_op(op.node, &result_ty);

        match lowered {
            LoweredBinOp::Unchecked(intrinsic_op) => {
                let temp = self.allocate_temp(result_ty);
                let place = ir::Place::Local(temp);
                let rvalue = ir::Rvalue::Intrinsic(
                    intrinsic_op,
                    vec![lhs_operand, rhs_operand],
                );
                self.emit_assign(place, rvalue);
                Ok(ir::Operand::Place(ir::Place::Local(temp)))
            }
            LoweredBinOp::Checked(intrinsic_op) => {
                self.lower_checked_binary_op(
                    intrinsic_op,
                    lhs_operand,
                    rhs_operand,
                    result_ty,
                )
            }
        }
    }

    /// Lower a binary operation directly into a destination place
    pub(crate) fn lower_binary_op_into(
        &mut self,
        binary_expr: &Expr<'db>,
        dest: ir::Place<'db>,
    ) -> MResult<()> {
        let (op, lhs, rhs) = match &binary_expr.kind {
            scrap_ast::expr::ExprKind::Binary(o, l, r) => (o, l, r),
            _ => return Err(crate::BuilderError::LowerExpressionError),
        };

        let lhs_operand = self.lower_expr(lhs)?;
        let rhs_operand = self.lower_expr(rhs)?;
        let result_ty = self.lookup_and_convert_type(binary_expr.id);
        let lowered = self.classify_bin_op(op.node, &result_ty);

        match lowered {
            LoweredBinOp::Unchecked(intrinsic_op) => {
                let rvalue = ir::Rvalue::Intrinsic(
                    intrinsic_op,
                    vec![lhs_operand, rhs_operand],
                );
                self.emit_assign(dest, rvalue);
            }
            LoweredBinOp::Checked(intrinsic_op) => {
                // Checked ops produce a tuple; lower via helper, then copy result to dest
                let result = self.lower_checked_binary_op(
                    intrinsic_op,
                    lhs_operand,
                    rhs_operand,
                    result_ty,
                )?;
                self.emit_assign(dest, ir::Rvalue::Use(result));
            }
        }
        Ok(())
    }

    /// Emit a checked binary operation with Assert terminator.
    ///
    /// Produces:
    /// ```text
    ///   _pair = intrinsic(lhs, rhs);  // (T, bool)
    ///   assert(!_pair.1, msg) -> success_bb;
    /// success_bb:
    ///   _result = _pair.0;
    /// ```
    pub(crate) fn lower_checked_binary_op(
        &mut self,
        intrinsic_op: ir::IntrinsicOp,
        lhs_operand: ir::Operand<'db>,
        rhs_operand: ir::Operand<'db>,
        result_ty: ir::Ty<'db>,
    ) -> MResult<ir::Operand<'db>> {
        // Allocate temp for the (T, bool) pair
        let pair_ty = ir::Ty::Tuple(vec![result_ty.clone(), ir::Ty::Bool]);
        let pair_temp = self.allocate_temp(pair_ty);

        // Emit: _pair = intrinsic(lhs, rhs)
        let pair_place = ir::Place::Local(pair_temp);
        let rvalue = ir::Rvalue::Intrinsic(
            intrinsic_op,
            vec![lhs_operand, rhs_operand],
        );
        self.emit_assign(pair_place, rvalue);

        // Extract the overflow flag: _pair.1
        let overflow_flag = ir::Operand::Place(ir::Place::Field(
            Box::new(ir::Place::Local(pair_temp)),
            1,
        ));

        // Create continuation block
        let success_bb = self.cfg_builder.start_block();

        // Determine assert message
        let msg = self.assert_message_for(intrinsic_op);

        // Finish current block with Assert
        self.cfg_builder.finish_block(ir::Terminator::Assert {
            cond: overflow_flag,
            expected: false, // expect overflow to be false
            msg,
            target: success_bb,
        });

        // Switch to success block
        self.cfg_builder.set_current_block(success_bb);

        // Extract the value: _pair.0
        let result_temp = self.allocate_temp(result_ty);
        let value_operand = ir::Operand::Place(ir::Place::Field(
            Box::new(ir::Place::Local(pair_temp)),
            0,
        ));
        self.emit_assign(
            ir::Place::Local(result_temp),
            ir::Rvalue::Use(value_operand),
        );

        Ok(ir::Operand::Place(ir::Place::Local(result_temp)))
    }

    /// Classify a binary operator as checked or unchecked based on the result type.
    ///
    /// Integer arithmetic ops are checked (overflow detection).
    /// Float ops, comparisons, logical, and bitwise ops are unchecked.
    fn classify_bin_op(
        &self,
        op: BinOpKind,
        result_ty: &ir::Ty<'db>,
    ) -> LoweredBinOp {
        let is_integer = matches!(result_ty, ir::Ty::Int(_) | ir::Ty::Uint(_));

        match op {
            // Arithmetic — checked for integers, unchecked for floats
            BinOpKind::Add if is_integer => LoweredBinOp::Checked(ir::IntrinsicOp::AddWithOverflow),
            BinOpKind::Sub if is_integer => LoweredBinOp::Checked(ir::IntrinsicOp::SubWithOverflow),
            BinOpKind::Mul if is_integer => LoweredBinOp::Checked(ir::IntrinsicOp::MulWithOverflow),
            BinOpKind::Div if is_integer => LoweredBinOp::Checked(ir::IntrinsicOp::DivWithZeroCheck),
            BinOpKind::Rem if is_integer => LoweredBinOp::Checked(ir::IntrinsicOp::RemWithZeroCheck),

            // Shifts — checked for integers
            BinOpKind::Shl if is_integer => LoweredBinOp::Checked(ir::IntrinsicOp::ShlChecked),
            BinOpKind::Shr if is_integer => LoweredBinOp::Checked(ir::IntrinsicOp::ShrChecked),

            // Unchecked arithmetic (floats, or fallback)
            BinOpKind::Add => LoweredBinOp::Unchecked(ir::IntrinsicOp::Add),
            BinOpKind::Sub => LoweredBinOp::Unchecked(ir::IntrinsicOp::Sub),
            BinOpKind::Mul => LoweredBinOp::Unchecked(ir::IntrinsicOp::Mul),
            BinOpKind::Div => LoweredBinOp::Unchecked(ir::IntrinsicOp::Div),
            BinOpKind::Rem => LoweredBinOp::Unchecked(ir::IntrinsicOp::Rem),

            // Shifts (non-integer, shouldn't really happen, but handle gracefully)
            BinOpKind::Shl => LoweredBinOp::Unchecked(ir::IntrinsicOp::Shl),
            BinOpKind::Shr => LoweredBinOp::Unchecked(ir::IntrinsicOp::Shr),

            // Comparisons — always unchecked
            BinOpKind::Eq => LoweredBinOp::Unchecked(ir::IntrinsicOp::Eq),
            BinOpKind::Ne => LoweredBinOp::Unchecked(ir::IntrinsicOp::Ne),
            BinOpKind::Lt => LoweredBinOp::Unchecked(ir::IntrinsicOp::Lt),
            BinOpKind::Le => LoweredBinOp::Unchecked(ir::IntrinsicOp::Le),
            BinOpKind::Gt => LoweredBinOp::Unchecked(ir::IntrinsicOp::Gt),
            BinOpKind::Ge => LoweredBinOp::Unchecked(ir::IntrinsicOp::Ge),

            // Logical — always unchecked
            BinOpKind::And => LoweredBinOp::Unchecked(ir::IntrinsicOp::And),
            BinOpKind::Or => LoweredBinOp::Unchecked(ir::IntrinsicOp::Or),

            // Bitwise — always unchecked
            BinOpKind::BitAnd => LoweredBinOp::Unchecked(ir::IntrinsicOp::BitAnd),
            BinOpKind::BitOr => LoweredBinOp::Unchecked(ir::IntrinsicOp::BitOr),
            BinOpKind::BitXor => LoweredBinOp::Unchecked(ir::IntrinsicOp::BitXor),
        }
    }

    /// Map an intrinsic op to its assert message.
    fn assert_message_for(&self, op: ir::IntrinsicOp) -> ir::AssertMessage {
        match op {
            ir::IntrinsicOp::AddWithOverflow
            | ir::IntrinsicOp::SubWithOverflow
            | ir::IntrinsicOp::MulWithOverflow => ir::AssertMessage::Overflow(op),
            ir::IntrinsicOp::DivWithZeroCheck => ir::AssertMessage::DivisionByZero,
            ir::IntrinsicOp::RemWithZeroCheck => ir::AssertMessage::RemainderByZero,
            ir::IntrinsicOp::ShlChecked | ir::IntrinsicOp::ShrChecked => {
                ir::AssertMessage::ShiftOverflow
            }
            _ => ir::AssertMessage::Overflow(op),
        }
    }

    /// Convert AST binary operator to IR intrinsic operator (unchecked).
    /// Used by tests and contexts where we always want the unchecked variant.
    pub(crate) fn convert_bin_op(&self, op: BinOpKind) -> MResult<ir::IntrinsicOp> {
        match op {
            BinOpKind::Add => Ok(ir::IntrinsicOp::Add),
            BinOpKind::Sub => Ok(ir::IntrinsicOp::Sub),
            BinOpKind::Mul => Ok(ir::IntrinsicOp::Mul),
            BinOpKind::Div => Ok(ir::IntrinsicOp::Div),
            BinOpKind::Rem => Ok(ir::IntrinsicOp::Rem),
            BinOpKind::And => Ok(ir::IntrinsicOp::And),
            BinOpKind::Or => Ok(ir::IntrinsicOp::Or),
            BinOpKind::BitXor => Ok(ir::IntrinsicOp::BitXor),
            BinOpKind::BitAnd => Ok(ir::IntrinsicOp::BitAnd),
            BinOpKind::BitOr => Ok(ir::IntrinsicOp::BitOr),
            BinOpKind::Shl => Ok(ir::IntrinsicOp::Shl),
            BinOpKind::Shr => Ok(ir::IntrinsicOp::Shr),
            BinOpKind::Eq => Ok(ir::IntrinsicOp::Eq),
            BinOpKind::Lt => Ok(ir::IntrinsicOp::Lt),
            BinOpKind::Le => Ok(ir::IntrinsicOp::Le),
            BinOpKind::Ne => Ok(ir::IntrinsicOp::Ne),
            BinOpKind::Ge => Ok(ir::IntrinsicOp::Ge),
            BinOpKind::Gt => Ok(ir::IntrinsicOp::Gt),
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

        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_sub(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 10);
        let rhs = create_int_lit(db, 4);
        let expr = create_binary_expr(db, BinOpKind::Sub, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_mul(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 6);
        let rhs = create_int_lit(db, 7);
        let expr = create_binary_expr(db, BinOpKind::Mul, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_binary_comparison(db: &dyn scrap_shared::Db) {
        let lhs = create_int_lit(db, 5);
        let rhs = create_int_lit(db, 10);
        let expr = create_binary_expr(db, BinOpKind::Lt, lhs, rhs);

        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));
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

        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));
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

        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));
        let result = lowerer.lower_expr(&mul_expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_parenthesized(db: &dyn scrap_shared::Db) {
        let inner = create_int_lit(db, 42);
        let expr = create_paren_expr(db, inner);

        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));
        let result = lowerer.lower_expr(&expr);
        assert!(result.is_ok());

        // Parentheses don't add anything, just unwrap
        assert_eq!(lowerer.local_decls.len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_operator_conversion(db: &dyn scrap_shared::Db) {
        let lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Test all arithmetic operators
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Add).unwrap(), ir::IntrinsicOp::Add);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Sub).unwrap(), ir::IntrinsicOp::Sub);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Mul).unwrap(), ir::IntrinsicOp::Mul);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Div).unwrap(), ir::IntrinsicOp::Div);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Rem).unwrap(), ir::IntrinsicOp::Rem);

        // Test comparison operators
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Eq).unwrap(), ir::IntrinsicOp::Eq);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Lt).unwrap(), ir::IntrinsicOp::Lt);
        assert_eq!(lowerer.convert_bin_op(BinOpKind::Gt).unwrap(), ir::IntrinsicOp::Gt);
    }
}
