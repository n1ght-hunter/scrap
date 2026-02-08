//! Function call lowering

use scrap_ast::expr::Expr;
use scrap_ir as ir;

use crate::{lowerer::ExprLowerer, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower the common parts of a function call (func operand + args).
    fn lower_call_parts(
        &mut self,
        call_expr: &Expr<'db>,
    ) -> MResult<(ir::Operand<'db>, Vec<ir::Operand<'db>>)> {
        let (func, args) = match &call_expr.kind {
            scrap_ast::expr::ExprKind::Call(f, a) => (f, a),
            _ => return Err(crate::BuilderError::LowerExpressionError),
        };

        let func_operand = self.lower_expr(func)?;
        let mut arg_operands = Vec::new();
        for arg in args {
            arg_operands.push(self.lower_expr(arg)?);
        }
        Ok((func_operand, arg_operands))
    }

    /// Emit a call terminator writing the result to `destination`.
    fn emit_call(
        &mut self,
        func: ir::Operand<'db>,
        args: Vec<ir::Operand<'db>>,
        destination: ir::Place<'db>,
    ) {
        let cont_bb = self.cfg_builder.start_block();
        let terminator = ir::Terminator::Call {
            func,
            args,
            destination,
            target: cont_bb,
            unwind: ir::UnwindAction::Continue,
        };
        self.cfg_builder.finish_block(terminator);
        self.cfg_builder.set_current_block(cont_bb);
    }

    /// Check if a call expression is a `box(value)` builtin.
    fn is_box_call(&self, call_expr: &Expr<'db>) -> bool {
        if let scrap_ast::expr::ExprKind::Call(callee, _) = &call_expr.kind {
            if let scrap_ast::expr::ExprKind::Path(path) = &callee.kind {
                if let Some(seg) = path.single_segment() {
                    return seg.ident.name.text(self.db) == "box";
                }
            }
        }
        false
    }

    /// Lower a `box(value)` expression to `Rvalue::Box`.
    fn lower_box_call(
        &mut self,
        call_expr: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        let args = match &call_expr.kind {
            scrap_ast::expr::ExprKind::Call(_, a) => a,
            _ => return Err(crate::BuilderError::LowerExpressionError),
        };

        // box(value) takes exactly one argument
        let value_operand = self.lower_expr(&args[0])?;

        // The result type of box(value) is *T — extract the inner type T
        let result_ty = self.lookup_and_convert_type(call_expr.id);
        let inner_ty = match &result_ty {
            ir::Ty::Ptr(inner) => (**inner).clone(),
            ir::Ty::Ref(inner, _) => (**inner).clone(),
            _ => self.lookup_and_convert_type(args[0].id),
        };

        let result_temp = self.allocate_temp(result_ty);
        let destination = ir::Place::Local(result_temp);

        self.emit_assign(destination.clone(), ir::Rvalue::Box(inner_ty, value_operand));
        Ok(ir::Operand::Place(destination))
    }

    /// Lower a function call to an operand (allocates a temporary).
    pub(crate) fn lower_call(
        &mut self,
        call_expr: &Expr<'db>,
    ) -> MResult<ir::Operand<'db>> {
        // Intercept box(value) builtin
        if self.is_box_call(call_expr) {
            return self.lower_box_call(call_expr);
        }

        let (func_operand, arg_operands) = self.lower_call_parts(call_expr)?;

        let result_ty = self.lookup_and_convert_type(call_expr.id);
        let result_temp = self.allocate_temp(result_ty);
        let destination = ir::Place::Local(result_temp);

        self.emit_call(func_operand, arg_operands, destination.clone());
        Ok(ir::Operand::Place(destination))
    }

    /// Lower a function call directly into a destination place.
    pub(crate) fn lower_call_into(
        &mut self,
        call_expr: &Expr<'db>,
        dest: ir::Place<'db>,
    ) -> MResult<()> {
        // Intercept box(value) builtin
        if self.is_box_call(call_expr) {
            let operand = self.lower_box_call(call_expr)?;
            self.emit_assign(dest, ir::Rvalue::Use(operand));
            return Ok(());
        }

        let (func_operand, arg_operands) = self.lower_call_parts(call_expr)?;
        self.emit_call(func_operand, arg_operands, dest);
        Ok(())
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
    fn test_lower_simple_call(db: &dyn scrap_shared::Db) {
        // foo()
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Create binding for foo
        let foo_sym = Symbol::new(db, "foo".to_string());
        let foo_local = lowerer.allocate_named_local(foo_sym, ir::Ty::Never);
        lowerer.insert_binding(foo_sym, foo_local);

        let func = create_ident_expr(db, "foo");
        let call_expr = create_call_expr(db, func, vec![]);

        let result = lowerer.lower_expr(&call_expr);
        assert!(result.is_ok());

        // Should have: foo, result_temp = 2 locals
        assert_eq!(lowerer.local_decls.len(), 2);

        // Should have created at least 2 blocks (call block + continuation)
        assert!(lowerer.cfg_builder.block_count() >= 2);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_with_args(db: &dyn scrap_shared::Db) {
        // add(1, 2)
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create binding for add
        let add_sym = Symbol::new(db, "add".to_string());
        let add_local = lowerer.allocate_named_local(add_sym, ir::Ty::Never);
        lowerer.insert_binding(add_sym, add_local);

        let func = create_ident_expr(db, "add");
        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let call_expr = create_call_expr(db, func, vec![one, two]);

        let result = lowerer.lower_expr(&call_expr);
        assert!(result.is_ok());

        // Should have: add, result_temp = 2 locals (literals are constants)
        assert_eq!(lowerer.local_decls.len(), 2);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_with_expression_args(db: &dyn scrap_shared::Db) {
        // max(x + 1, y * 2)
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create bindings
        let max_sym = Symbol::new(db, "max".to_string());
        let max_local = lowerer.allocate_named_local(max_sym, ir::Ty::Never);
        lowerer.insert_binding(max_sym, max_local);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let y_sym = Symbol::new(db, "y".to_string());
        let y_local = lowerer.allocate_named_local(y_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(y_sym, y_local);

        // Create the call: max(x + 1, y * 2)
        let func = create_ident_expr(db, "max");
        let x_expr = create_ident_expr(db, "x");
        let one = create_int_lit(db, 1);
        let arg1 = create_binary_expr(db, BinOpKind::Add, x_expr, one);

        let y_expr = create_ident_expr(db, "y");
        let two = create_int_lit(db, 2);
        let arg2 = create_binary_expr(db, BinOpKind::Mul, y_expr, two);

        let call_expr = create_call_expr(db, func, vec![arg1, arg2]);

        let result = lowerer.lower_expr(&call_expr);
        assert!(result.is_ok());

        // Should have: max, x, y, pair1, add_result, pair2, mul_result, call_result = 8 locals
        assert_eq!(lowerer.local_decls.len(), 8);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_nested_calls(db: &dyn scrap_shared::Db) {
        // outer(inner(1))
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create bindings
        let outer_sym = Symbol::new(db, "outer".to_string());
        let outer_local = lowerer.allocate_named_local(outer_sym, ir::Ty::Never);
        lowerer.insert_binding(outer_sym, outer_local);

        let inner_sym = Symbol::new(db, "inner".to_string());
        let inner_local = lowerer.allocate_named_local(inner_sym, ir::Ty::Never);
        lowerer.insert_binding(inner_sym, inner_local);

        // Create inner call: inner(1)
        let inner_func = create_ident_expr(db, "inner");
        let one = create_int_lit(db, 1);
        let inner_call = create_call_expr(db, inner_func, vec![one]);

        // Create outer call: outer(inner(1))
        let outer_func = create_ident_expr(db, "outer");
        let outer_call = create_call_expr(db, outer_func, vec![inner_call]);

        let result = lowerer.lower_expr(&outer_call);
        assert!(result.is_ok());

        // Should have: outer, inner, inner_result, outer_result = 4 locals
        assert_eq!(lowerer.local_decls.len(), 4);

        // Should have multiple blocks for nested calls
        assert!(lowerer.cfg_builder.block_count() >= 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_result_assignment(db: &dyn scrap_shared::Db) {
        // result = foo(1, 2)
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create bindings
        let result_sym = Symbol::new(db, "result".to_string());
        let result_local = lowerer.allocate_named_local(result_sym, ir::Ty::Never);
        lowerer.insert_binding(result_sym, result_local);

        let foo_sym = Symbol::new(db, "foo".to_string());
        let foo_local = lowerer.allocate_named_local(foo_sym, ir::Ty::Never);
        lowerer.insert_binding(foo_sym, foo_local);

        // Create call: foo(1, 2)
        let func = create_ident_expr(db, "foo");
        let one = create_int_lit(db, 1);
        let two = create_int_lit(db, 2);
        let call_expr = create_call_expr(db, func, vec![one, two]);

        // Create assignment: result = foo(1, 2)
        let result_expr = create_ident_expr(db, "result");
        let assign_expr = create_assign_expr(db, result_expr, call_expr);

        let result = lowerer.lower_expr(&assign_expr);
        assert!(result.is_ok());

        // Should have: result, foo, call_result = 3 locals
        assert_eq!(lowerer.local_decls.len(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_call_in_if_condition(db: &dyn scrap_shared::Db) {
        // if is_valid(x) { }
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        // Create bindings
        let is_valid_sym = Symbol::new(db, "is_valid".to_string());
        let is_valid_local = lowerer.allocate_named_local(is_valid_sym, ir::Ty::Never);
        lowerer.insert_binding(is_valid_sym, is_valid_local);

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // Create call: is_valid(x)
        let func = create_ident_expr(db, "is_valid");
        let x_expr = create_ident_expr(db, "x");
        let call_expr = create_call_expr(db, func, vec![x_expr]);

        // Create if: if is_valid(x) { }
        let then_block = create_empty_block(db);
        let if_expr = create_if_expr(db, call_expr, then_block);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Should have multiple blocks for call + if structure
        assert!(lowerer.cfg_builder.block_count() >= 5);
    }
}
