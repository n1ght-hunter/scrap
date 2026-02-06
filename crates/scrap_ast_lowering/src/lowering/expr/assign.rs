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
            _ => {
                // Only paths can be places for now
                // Future: field access, array indexing, etc.
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
    /// Desugars to: lhs = lhs op rhs
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

        // Convert the assignment operator to a binary operator
        let bin_op = self.convert_assign_op(op.node)?;

        // Create the binary operation: lhs op rhs
        let rvalue = ir::Rvalue::BinaryOp(bin_op, lhs_operand, rhs_operand);

        // Emit the assignment: place = lhs op rhs
        self.emit_assign(place, rvalue);

        // Assignments produce void
        Ok(ir::Operand::Constant(ir::Constant::Void))
    }

    /// Convert AST assignment operator to IR binary operator
    pub(crate) fn convert_assign_op(&self, op: AssignOpKind) -> MResult<ir::BinOp> {
        match op {
            AssignOpKind::AddAssign => Ok(ir::BinOp::Add),
            AssignOpKind::SubAssign => Ok(ir::BinOp::Sub),
            AssignOpKind::MulAssign => Ok(ir::BinOp::Mul),
            AssignOpKind::DivAssign => Ok(ir::BinOp::Div),
            AssignOpKind::RemAssign => Ok(ir::BinOp::Rem),
            AssignOpKind::BitXorAssign => Ok(ir::BinOp::BitXor),
            AssignOpKind::BitAndAssign => Ok(ir::BinOp::BitAnd),
            AssignOpKind::BitOrAssign => Ok(ir::BinOp::BitOr),
            AssignOpKind::ShlAssign => Ok(ir::BinOp::Shl),
            AssignOpKind::ShrAssign => Ok(ir::BinOp::Shr),
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
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

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

        // Should have 2 locals: x, and temp for 5
        assert_eq!(lowerer.local_decls.len(), 2);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_assignment_to_undefined_variable(db: &dyn scrap_shared::Db) {
        // undefined = 5 (should fail)
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let lhs = create_ident_expr(db, "undefined");
        let rhs = create_int_lit(db, 5);
        let assign_expr = create_assign_expr(db, lhs, rhs);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_err());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_add(db: &dyn scrap_shared::Db) {
        // x += 5
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

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

        // Should have 2 locals: x, temp for 5
        assert_eq!(lowerer.local_decls.len(), 2);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_compound_assignment_sub(db: &dyn scrap_shared::Db) {
        // x -= 3
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

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
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

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
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

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
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

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

        // Should have: x, 5_temp, 3_temp, add_result_temp
        assert_eq!(lowerer.local_decls.len(), 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_chained_assignment_operations(db: &dyn scrap_shared::Db) {
        // x += 5; then x *= 2 (two separate operations)
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

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
        // x, 5_temp, 2_temp
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_assign_op_conversion(db: &dyn scrap_shared::Db) {
        let lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Test all assignment operators
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::AddAssign).unwrap(), ir::BinOp::Add);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::SubAssign).unwrap(), ir::BinOp::Sub);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::MulAssign).unwrap(), ir::BinOp::Mul);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::DivAssign).unwrap(), ir::BinOp::Div);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::RemAssign).unwrap(), ir::BinOp::Rem);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::BitXorAssign).unwrap(), ir::BinOp::BitXor);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::BitAndAssign).unwrap(), ir::BinOp::BitAnd);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::BitOrAssign).unwrap(), ir::BinOp::BitOr);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::ShlAssign).unwrap(), ir::BinOp::Shl);
        assert_eq!(lowerer.convert_assign_op(AssignOpKind::ShrAssign).unwrap(), ir::BinOp::Shr);
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
