//! Assignment lowering

use scrap_ast::{
    expr::{Expr, ExprKind},
    operators::{AssignOp, AssignOpKind},
};
use scrap_ir as ir;

use crate::{lowerer::ExprLowerer, BuilderError, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower an expression to a place (for use as LHS of assignment)
    pub(crate) fn lower_place(&mut self, expr: &Expr<'db>) -> MResult<ir::Place<'db>> {
        match &expr.kind {
            ExprKind::Path(path) => {
                // Extract the identifier from the path
                let ident = path
                    .single_segment()
                    .ok_or(BuilderError::LowerExpressionError)?
                    .ident;

                // Look up the variable in the symbol table
                let local_id = self
                    .lookup_binding(ident.name)
                    .ok_or(BuilderError::LowerExpressionError)?;

                Ok(ir::Place::Local(local_id))
            }
            ExprKind::Unary(scrap_ast::operators::UnOp::Deref, inner) => {
                // *x as a place — dereference through a reference
                let inner_operand = self.lower_expr(inner)?;
                let inner_place = match inner_operand {
                    ir::Operand::Place(place) => place,
                    other => {
                        let ref_ty = self.lookup_and_convert_type(inner.id);
                        let temp = self.allocate_temp(ref_ty);
                        self.emit_assign(ir::Place::Local(temp), ir::Rvalue::Use(other));
                        ir::Place::Local(temp)
                    }
                };
                Ok(ir::Place::Deref(Box::new(inner_place)))
            }
            _ => {
                // Only paths and dereferences can be places for now
                Err(BuilderError::LowerExpressionError)
            }
        }
    }

    /// Lower an assignment expression: lhs = rhs
    pub(crate) fn lower_assign(
        &mut self,
        lhs: &Expr<'db>,
        rhs: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        // Lower the LHS to a place
        let place = self.lower_place(lhs)?;

        // Lower the RHS to an operand
        let rhs_operand = self.lower_expr(rhs)?;

        // Emit the assignment: place = Use(rhs_operand)
        let rvalue = ir::Rvalue::Use(rhs_operand);
        self.emit_assign(place, rvalue);

        // Assignments produce void
        Ok(ir::Operand::Constant(ir::Constant::Void))
    }

    /// Lower a compound assignment expression: lhs op= rhs
    /// Desugars to: lhs = lhs op rhs (with checked arithmetic for integers)
    pub(crate) fn lower_assign_op(
        &mut self,
        op: &AssignOp<'db>,
        lhs: &Expr<'db>,
        rhs: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        // Lower the LHS to a place
        let place = self.lower_place(lhs)?;

        // Lower the RHS to an operand
        let rhs_operand = self.lower_expr(rhs)?;

        // Create an operand for the LHS (to read its current value)
        let lhs_operand = ir::Operand::Place(place.clone());

        // Get the type of the LHS to determine checked vs unchecked
        let lhs_ty = self.lookup_and_convert_type(lhs.id);
        let is_integer = matches!(lhs_ty, ir::Ty::Int(_) | ir::Ty::Uint(_));

        // Convert the assignment operator to an intrinsic op
        let intrinsic_op = self.convert_assign_op(op.node, is_integer)?;

        if is_integer && Self::is_checked_intrinsic(intrinsic_op) {
            // Checked path: produce (T, bool), assert, extract value
            let result = self.lower_checked_binary_op(
                intrinsic_op,
                lhs_operand,
                rhs_operand,
                lhs_ty,
            )?;
            self.emit_assign(place, ir::Rvalue::Use(result));
        } else {
            // Unchecked path: direct intrinsic call
            let rvalue = ir::Rvalue::Intrinsic(intrinsic_op, vec![lhs_operand, rhs_operand]);
            self.emit_assign(place, rvalue);
        }

        // Assignments produce void
        Ok(ir::Operand::Constant(ir::Constant::Void))
    }

    /// Check if an intrinsic op is a checked variant.
    fn is_checked_intrinsic(op: ir::IntrinsicOp) -> bool {
        matches!(
            op,
            ir::IntrinsicOp::AddWithOverflow
                | ir::IntrinsicOp::SubWithOverflow
                | ir::IntrinsicOp::MulWithOverflow
                | ir::IntrinsicOp::DivWithZeroCheck
                | ir::IntrinsicOp::RemWithZeroCheck
                | ir::IntrinsicOp::ShlChecked
                | ir::IntrinsicOp::ShrChecked
        )
    }

    /// Convert AST assignment operator to IR intrinsic operator.
    /// Returns the checked variant for integer types, unchecked for others.
    pub(crate) fn convert_assign_op(
        &self,
        op: AssignOpKind,
        is_integer: bool,
    ) -> MResult<ir::IntrinsicOp> {
        match op {
            AssignOpKind::AddAssign if is_integer => Ok(ir::IntrinsicOp::AddWithOverflow),
            AssignOpKind::SubAssign if is_integer => Ok(ir::IntrinsicOp::SubWithOverflow),
            AssignOpKind::MulAssign if is_integer => Ok(ir::IntrinsicOp::MulWithOverflow),
            AssignOpKind::DivAssign if is_integer => Ok(ir::IntrinsicOp::DivWithZeroCheck),
            AssignOpKind::RemAssign if is_integer => Ok(ir::IntrinsicOp::RemWithZeroCheck),
            AssignOpKind::ShlAssign if is_integer => Ok(ir::IntrinsicOp::ShlChecked),
            AssignOpKind::ShrAssign if is_integer => Ok(ir::IntrinsicOp::ShrChecked),

            AssignOpKind::AddAssign => Ok(ir::IntrinsicOp::Add),
            AssignOpKind::SubAssign => Ok(ir::IntrinsicOp::Sub),
            AssignOpKind::MulAssign => Ok(ir::IntrinsicOp::Mul),
            AssignOpKind::DivAssign => Ok(ir::IntrinsicOp::Div),
            AssignOpKind::RemAssign => Ok(ir::IntrinsicOp::Rem),
            AssignOpKind::BitXorAssign => Ok(ir::IntrinsicOp::BitXor),
            AssignOpKind::BitAndAssign => Ok(ir::IntrinsicOp::BitAnd),
            AssignOpKind::BitOrAssign => Ok(ir::IntrinsicOp::BitOr),
            AssignOpKind::ShlAssign => Ok(ir::IntrinsicOp::Shl),
            AssignOpKind::ShrAssign => Ok(ir::IntrinsicOp::Shr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use scrap_ast::operators::BinOpKind;
    use scrap_shared::ident::Symbol;
    use scrap_shared::types::IntTy;

    #[scrap_macros::salsa_test]
    fn test_lower_simple_assignment(db: &dyn scrap_shared::Db) {
        // x = 5
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // First, create a binding for "x"
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // Create the assignment: x = 5
        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 5);
        let assign_expr = create_assign_expr(db, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have 1 local: x (literals are constants, no temp)
        assert_eq!(lowerer.local_decls.len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_assignment_to_undefined_variable(db: &dyn scrap_shared::Db) {
        // undefined = 5 (should fail)
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let lhs = create_ident_expr(db, "undefined");
        let rhs = create_int_lit(db, 5);
        let assign_expr = create_assign_expr(db, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_err());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_add(db: &dyn scrap_shared::Db) {
        // x += 5
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create a binding for "x"
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // Create the assignment: x += 5
        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 5);
        let assign_expr = create_assign_op_expr(db, AssignOpKind::AddAssign, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have 3 locals: x, tuple pair (i32, bool), extracted result
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_sub(db: &dyn scrap_shared::Db) {
        // x -= 3
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 3);
        let assign_expr = create_assign_op_expr(db, AssignOpKind::SubAssign, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_mul(db: &dyn scrap_shared::Db) {
        // x *= 2
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 2);
        let assign_expr = create_assign_op_expr(db, AssignOpKind::MulAssign, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_bitwise(db: &dyn scrap_shared::Db) {
        // x <<= 1
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 1);
        let assign_expr = create_assign_op_expr(db, AssignOpKind::ShlAssign, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_assignment_with_expression(db: &dyn scrap_shared::Db) {
        // x = 5 + 3
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let five = create_int_lit(db, 5);
        let three = create_int_lit(db, 3);
        let add_expr = create_binary_expr(db, BinOpKind::Add, five, three);
        let assign_expr = create_assign_expr(db, lhs, add_expr);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have: x, tuple pair (i32, bool), add_result_temp
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_chained_assignment_operations(db: &dyn scrap_shared::Db) {
        // x += 5; then x *= 2 (two separate operations)
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // First: x += 5
        let lhs1 = create_ident_expr(db, "x");
        let rhs1 = create_int_lit(db, 5);
        let assign1 = create_assign_op_expr(db, AssignOpKind::AddAssign, lhs1, rhs1);
        let result1 = lowerer.lower_expr(&assign1);
        assert!(result1.is_ok());

        // Second: x *= 2
        let lhs2 = create_ident_expr(db, "x");
        let rhs2 = create_int_lit(db, 2);
        let assign2 = create_assign_op_expr(db, AssignOpKind::MulAssign, lhs2, rhs2);
        let result2 = lowerer.lower_expr(&assign2);
        assert!(result2.is_ok());

        // Should have accumulated locals from both operations
        // x, pair1, result1, pair2, result2
        assert_eq!(lowerer.local_decls.len(), 5);
    }

    #[scrap_macros::salsa_test]
    fn test_assign_op_conversion(db: &dyn scrap_shared::Db) {
        let lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Test unchecked (non-integer) assignment operators
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::AddAssign, false).unwrap(), ir::IntrinsicOp::Add);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::SubAssign, false).unwrap(), ir::IntrinsicOp::Sub);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::MulAssign, false).unwrap(), ir::IntrinsicOp::Mul);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::DivAssign, false).unwrap(), ir::IntrinsicOp::Div);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::RemAssign, false).unwrap(), ir::IntrinsicOp::Rem);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::BitXorAssign, false).unwrap(), ir::IntrinsicOp::BitXor);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::BitAndAssign, false).unwrap(), ir::IntrinsicOp::BitAnd);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::BitOrAssign, false).unwrap(), ir::IntrinsicOp::BitOr);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::ShlAssign, false).unwrap(), ir::IntrinsicOp::Shl);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::ShrAssign, false).unwrap(), ir::IntrinsicOp::Shr);

        // Test checked (integer) assignment operators
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::AddAssign, true).unwrap(), ir::IntrinsicOp::AddWithOverflow);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::SubAssign, true).unwrap(), ir::IntrinsicOp::SubWithOverflow);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::MulAssign, true).unwrap(), ir::IntrinsicOp::MulWithOverflow);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_place_from_path(db: &dyn scrap_shared::Db) {
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let expr = create_ident_expr(db, "x");
        let result = lowerer.lower_place(&expr);
        assert!(result.is_ok());

        let place = result.unwrap();
        assert!(matches!(place, ir::Place::Local(_)));
    }

    #[scrap_macros::salsa_test]
    fn test_lower_place_from_literal_fails(db: &dyn scrap_shared::Db) {
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Trying to use a literal as an lvalue should fail
        let expr = create_int_lit(db, 42);
        let result = lowerer.lower_place(&expr);
        assert!(result.is_err());
    }
}
