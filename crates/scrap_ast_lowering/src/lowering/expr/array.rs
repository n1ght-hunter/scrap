//! Array lowering

use scrap_ast::expr::Expr;
use scrap_ir as ir;

use crate::{lowerer::ExprLowerer, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower an array literal to an operand
    pub(crate) fn lower_array(&mut self, array_expr: &Expr<'db>) -> MResult<ir::Operand<'db>> {
        // Extract elements from array expression
        let elements = match &array_expr.kind {
            scrap_ast::expr::ExprKind::Array(elems) => elems,
            _ => return Err(crate::BuilderError::LowerExpressionError),
        };

        // Lower each element to an operand
        let mut element_operands = Vec::new();
        for element in elements {
            let operand = self.lower_expr(element)?;
            element_operands.push(operand);
        }

        // Allocate a temporary for the array using type from type table
        let result_ty = self.lookup_and_convert_type(array_expr.id);
        let temp = self.allocate_temp(result_ty);

        // Emit assignment: temp = [elem1, elem2, ...]
        let place = ir::Place::Local(temp);
        let rvalue = ir::Rvalue::Array(element_operands);
        self.emit_assign(place, rvalue);

        // Return reference to the array temporary
        Ok(ir::Operand::Place(ir::Place::Local(temp)))
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
    fn test_lower_empty_array(db: &dyn scrap_shared::Db) {
        // []
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let array_expr = create_array_expr(db, vec![]);

        let result = lowerer.lower_expr(&array_expr);
        assert!(result.is_ok());

        // Should have created one temporary for the array
        assert_eq!(lowerer.local_decls.len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_array_with_literals(db: &dyn scrap_shared::Db) {
        // [1, 2, 3]
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let three = create_int_lit(db, 3);
        let array_expr = create_array_expr(db, vec![one, two, three]);

        let result = lowerer.lower_expr(&array_expr);
        assert!(result.is_ok());

        // Should have: 1_temp, 2_temp, 3_temp, array_temp = 4 locals
        assert_eq!(lowerer.local_decls.len(), 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_array_with_variables(db: &dyn scrap_shared::Db) {
        // [x, y]
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Create bindings for x and y
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let y_sym = Symbol::new(db, "y".to_string());
        let y_local = lowerer.allocate_named_local(y_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(y_sym, y_local);

        // Create the array
        let x_expr = create_ident_expr(db, "x");
        let y_expr = create_ident_expr(db, "y");
        let array_expr = create_array_expr(db, vec![x_expr, y_expr]);

        let result = lowerer.lower_expr(&array_expr);
        assert!(result.is_ok());

        // Should have: x, y, array_temp = 3 locals
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_array_with_expressions(db: &dyn scrap_shared::Db) {
        // [1 + 2, 3 * 4]
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let add_expr = create_binary_expr(db, BinOpKind::Add, one, two);

        let three = create_int_lit(db, 3);
        let four = create_int_lit(db, 4);
        let mul_expr = create_binary_expr(db, BinOpKind::Mul, three, four);

        let array_expr = create_array_expr(db, vec![add_expr, mul_expr]);

        let result = lowerer.lower_expr(&array_expr);
        assert!(result.is_ok());

        // Should have: 1, 2, add_result, 3, 4, mul_result, array = 7 locals
        assert_eq!(lowerer.local_decls.len(), 7);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_nested_array(db: &dyn scrap_shared::Db) {
        // [[1, 2], [3, 4]]
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let inner1 = create_array_expr(db, vec![one, two]);

        let three = create_int_lit(db, 3);
        let four = create_int_lit(db, 4);
        let inner2 = create_array_expr(db, vec![three, four]);

        let outer = create_array_expr(db, vec![inner1, inner2]);

        let result = lowerer.lower_expr(&outer);
        assert!(result.is_ok());

        // Should have: 1, 2, inner1_array, 3, 4, inner2_array, outer_array = 7 locals
        assert_eq!(lowerer.local_decls.len(), 7);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_array_assignment(db: &dyn scrap_shared::Db) {
        // arr = [1, 2, 3]
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Create binding for arr
        let arr_sym = Symbol::new(db, "arr".to_string());
        let arr_local = lowerer.allocate_named_local(arr_sym, ir::Ty::Never);
        lowerer.insert_binding(arr_sym, arr_local);

        // Create the array
        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let three = create_int_lit(db, 3);
        let array_expr = create_array_expr(db, vec![one, two, three]);

        // Create the assignment
        let arr_expr = create_ident_expr(db, "arr");
        let assign_expr = create_assign_expr(db, arr_expr, array_expr);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have: arr, 1, 2, 3, array_temp = 5 locals
        assert_eq!(lowerer.local_decls.len(), 5);
    }
}
