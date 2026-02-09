//! Control flow lowering (if/else, return)

use scrap_ast::{block::Block, expr::Expr};
use scrap_ir as ir;

use crate::{lowerer::ExprLowerer, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower an if-expression with optional else
    pub(crate) fn lower_if_expr(
        &mut self,
        cond: &Expr<'db>,
        then_block: &Block<'db>,
        else_expr: Option<&Expr<'db>>,
        node_id: scrap_shared::NodeId,
    ) -> MResult<ir::Operand<'db>> {
        // Evaluate the condition in the current block
        let cond_operand = self.lower_expr(cond)?;

        // Allocate blocks for the CFG
        let then_bb = self.cfg_builder.start_block();
        let else_bb = self.cfg_builder.start_block();
        let cont_bb = self.cfg_builder.start_block();

        // Finish current block with SwitchInt
        // value 0 (false) → else_bb, otherwise (true) → then_bb
        let terminator = ir::Terminator::SwitchInt {
            discr: cond_operand,
            targets: ir::SwitchTargets {
                values: vec![(0, else_bb)],
                otherwise: then_bb,
            },
        };
        self.cfg_builder.finish_block(terminator);

        // If there's an else branch, this if-else is an expression that produces a value.
        // Allocate a result temp and write both branches into it.
        let has_else = else_expr.is_some();
        let result_temp = if has_else {
            let result_ty = self.lookup_and_convert_type(node_id);
            Some(self.allocate_temp(result_ty))
        } else {
            None
        };

        // Lower the then block
        self.cfg_builder.set_current_block(then_bb);
        let then_operand = self.lower_block(then_block)?;
        if !self.cfg_builder.current_block_is_terminated() {
            if let Some(result) = result_temp {
                self.emit_assign(
                    ir::Place::Local(result),
                    ir::Rvalue::Use(then_operand),
                );
            }
            self.cfg_builder
                .finish_block(ir::Terminator::Goto { target: cont_bb });
        }

        // Lower the else expression/block
        self.cfg_builder.set_current_block(else_bb);
        if let Some(else_expr) = else_expr {
            let else_operand = self.lower_expr(else_expr)?;
            if !self.cfg_builder.current_block_is_terminated() {
                if let Some(result) = result_temp {
                    self.emit_assign(
                        ir::Place::Local(result),
                        ir::Rvalue::Use(else_operand),
                    );
                }
            }
        }
        if !self.cfg_builder.current_block_is_terminated() {
            self.cfg_builder
                .finish_block(ir::Terminator::Goto { target: cont_bb });
        }

        // Continue at the continuation block
        self.cfg_builder.set_current_block(cont_bb);

        if let Some(result) = result_temp {
            Ok(ir::Operand::Place(ir::Place::Local(result)))
        } else {
            Ok(ir::Operand::Constant(ir::Constant::Void))
        }
    }

    /// Lower a return statement
    pub(crate) fn lower_return(&mut self, value: Option<&Expr<'db>>) -> MResult<ir::Operand<'db>> {
        // Assign return value directly to _0
        if let Some(expr) = value {
            let ret_place = self.return_place();
            self.lower_expr_into(expr, ret_place)?;
        }

        // Emit the return terminator
        self.cfg_builder.finish_block(ir::Terminator::Return);

        // Returns don't produce a value
        Ok(ir::Operand::Constant(ir::Constant::Void))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use scrap_ast::expr::ExprKind;
    use scrap_ast::operators::BinOpKind;
    use scrap_shared::ident::Symbol;
    use scrap_shared::types::IntTy;

    #[scrap_macros::salsa_test]
    fn test_lower_if_without_else(db: &dyn scrap_shared::Db) {
        // if x > 0 { }
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        // Create variable x
        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // Create condition: x > 0
        let x_expr = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero);

        // Create empty then block
        let then_block = create_empty_block(db);

        // Create if expression
        let if_expr = create_if_expr(db, cond, then_block);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Should have created multiple blocks for the CFG
        assert!(lowerer.cfg_builder.block_count() >= 4); // entry, then, else, cont
    }

    #[scrap_macros::salsa_test]
    fn test_lower_if_with_else(db: &dyn scrap_shared::Db) {
        // if x > 0 { } else { }
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // Condition: x > 0
        let x_expr = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero);

        // Then and else blocks
        let then_block = create_empty_block(db);
        let else_block = create_empty_block(db);
        let else_expr = Expr {
            id: test_node_id(),
            kind: ExprKind::Block(Box::new(else_block)),
            span: test_span(db),
        };

        let if_expr = create_if_else_expr(db, cond, then_block, else_expr);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Should have created CFG with multiple blocks
        assert!(lowerer.cfg_builder.block_count() >= 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_nested_if(db: &dyn scrap_shared::Db) {
        // if x > 0 { if y > 0 { } }
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let y_sym = Symbol::new(db, "y".to_string());
        let y_local = lowerer.allocate_named_local(y_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(y_sym, y_local);

        // Outer condition: x > 0
        let x_expr = create_ident_expr(db, "x");
        let zero1 = create_int_lit(db, 0);
        let outer_cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero1);

        // Inner condition: y > 0
        let y_expr = create_ident_expr(db, "y");
        let zero2 = create_int_lit(db, 0);
        let inner_cond = create_binary_expr(db, BinOpKind::Gt, y_expr, zero2);

        // Inner if
        let inner_then = create_empty_block(db);
        let inner_if = create_if_expr(db, inner_cond, inner_then);

        // Outer then block contains inner if
        let stmt = create_expr_stmt(db, inner_if);
        let outer_then = create_block(db, vec![stmt]);

        // Outer if
        let outer_if = create_if_expr(db, outer_cond, outer_then);

        let result = lowerer.lower_expr(&outer_if);
        assert!(result.is_ok());

        // Nested ifs create even more blocks
        assert!(lowerer.cfg_builder.block_count() >= 7);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_return_without_value(db: &dyn scrap_shared::Db) {
        // return;
        let mut lowerer = ExprLowerer::new(db, "", create_empty_type_table(db));

        let return_expr = create_return_expr(db, None);

        let result = lowerer.lower_expr(&return_expr);
        assert!(result.is_ok());

        // Current block should be terminated
        assert!(lowerer.cfg_builder.current_block_is_terminated());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_return_with_value(db: &dyn scrap_shared::Db) {
        // return 42;
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let value = create_int_lit(db, 42);
        let return_expr = create_return_expr(db, Some(value));

        let result = lowerer.lower_expr(&return_expr);
        assert!(result.is_ok());

        // lower_literal_into writes the constant directly to the return place (_0),
        // so no temporary local is allocated
        assert_eq!(lowerer.local_decls.len(), 0);

        // Current block should be terminated
        assert!(lowerer.cfg_builder.current_block_is_terminated());
    }

    #[scrap_macros::salsa_test]
    fn test_lower_if_with_return_in_then(db: &dyn scrap_shared::Db) {
        // if x > 0 { return 1; }
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // Condition
        let x_expr = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero);

        // Then block with return
        let one = create_int_lit(db, 1);
        let return_expr = create_return_expr(db, Some(one));
        let stmt = create_expr_stmt(db, return_expr);
        let then_block = create_block(db, vec![stmt]);

        let if_expr = create_if_expr(db, cond, then_block);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // CFG should have multiple blocks
        assert!(lowerer.cfg_builder.block_count() >= 4);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_early_return(db: &dyn scrap_shared::Db) {
        // return 42; (followed by more code would be dead)
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let value = create_int_lit(db, 42);
        let return_expr = create_return_expr(db, Some(value));

        lowerer.lower_expr(&return_expr).unwrap();

        // Try to add another expression (this should be dead code)
        let dead_expr = create_int_lit(db, 99);
        lowerer.lower_expr(&dead_expr).unwrap();

        // Should handle dead code gracefully
        assert!(lowerer.cfg_builder.block_count() >= 1);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_if_with_complex_condition(db: &dyn scrap_shared::Db) {
        // if x > 0 && y < 10 { }
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let y_sym = Symbol::new(db, "y".to_string());
        let y_local = lowerer.allocate_named_local(y_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(y_sym, y_local);

        // x > 0
        let x_expr = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let left_cond = create_binary_expr(db, BinOpKind::Gt, x_expr, zero);

        // y < 10
        let y_expr = create_ident_expr(db, "y");
        let ten = create_int_lit(db, 10);
        let right_cond = create_binary_expr(db, BinOpKind::Lt, y_expr, ten);

        // x > 0 && y < 10
        let cond = create_binary_expr(db, BinOpKind::And, left_cond, right_cond);

        let then_block = create_empty_block(db);
        let if_expr = create_if_expr(db, cond, then_block);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Should have created multiple temporaries for the complex condition
        assert!(lowerer.local_decls.len() >= 5); // x, y, 0, 10, and temps for comparisons and &&
    }

    #[scrap_macros::salsa_test]
    fn test_cfg_builder_integration(db: &dyn scrap_shared::Db) {
        // Test that we can build a complete CFG
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        // if x > 0 { return 1; } else { return 0; }
        let x_expr1 = create_ident_expr(db, "x");
        let zero = create_int_lit(db, 0);
        let cond = create_binary_expr(db, BinOpKind::Gt, x_expr1, zero);

        let one = create_int_lit(db, 1);
        let return_one = create_return_expr(db, Some(one));
        let then_stmt = create_expr_stmt(db, return_one);
        let then_block = create_block(db, vec![then_stmt]);

        let zero2 = create_int_lit(db, 0);
        let return_zero = create_return_expr(db, Some(zero2));
        let else_block_expr = Expr {
            id: test_node_id(),
            kind: ExprKind::Block(Box::new(create_block(db, vec![create_expr_stmt(db, return_zero)]))),
            span: test_span(db),
        };

        let if_expr = create_if_else_expr(db, cond, then_block, else_block_expr);

        let result = lowerer.lower_expr(&if_expr);
        assert!(result.is_ok());

        // Build the CFG
        let blocks = lowerer.cfg_builder.build();

        // Should have created blocks
        assert!(blocks.len() >= 4);

        // Check that each block has a terminator
        for block in &blocks {
            // Every block should have a terminator
            let term = block.terminator(db);
            assert!(!matches!(term, ir::Terminator::Unreachable) || block.statements(db).is_empty());
        }
    }
}
