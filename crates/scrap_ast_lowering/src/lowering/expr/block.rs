//! Block lowering

use scrap_ast::block::Block;
use scrap_ir as ir;

use crate::{MResult, lowerer::ExprLowerer};

impl<'db> ExprLowerer<'db> {
    /// Lower a block expression
    pub(crate) fn lower_block_expr(&mut self, block: &Block<'db>) -> MResult<ir::Operand<'db>> {
        self.lower_block(block)
    }

    /// Lower a block's statements, returning the result of the last expression.
    pub(crate) fn lower_block(&mut self, block: &Block<'db>) -> MResult<ir::Operand<'db>> {
        let mut last_operand = ir::Operand::Constant(ir::Constant::Void);

        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i == block.stmts.len() - 1;
            match &stmt.kind {
                scrap_ast::stmt::StmtKind::Expr(expr) if is_last => {
                    // Last expression without semicolon — this is the block's result
                    last_operand = self.lower_expr(expr)?;
                }
                scrap_ast::stmt::StmtKind::Semi(expr) | scrap_ast::stmt::StmtKind::Expr(expr) => {
                    self.lower_expr(expr)?;
                }
                scrap_ast::stmt::StmtKind::Let(_local) => {
                    // TODO: Handle let statements with initializers
                }
                scrap_ast::stmt::StmtKind::Item(_) | scrap_ast::stmt::StmtKind::Empty => {}
            }
        }

        Ok(last_operand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use scrap_ast::expr::{Expr, ExprKind};
    use scrap_shared::ident::Symbol;
    use scrap_shared::types::IntTy;

    #[scrap_macros::salsa_test]
    fn test_lower_block_expr(db: &dyn scrap_shared::Db) {
        // { x = 5; }
        let mut lowerer = ExprLowerer::new(db, TEST_SOURCE, create_test_type_table(db));

        let x_sym = Symbol::new(db, "x".to_string());
        let x_local = lowerer.allocate_named_local(x_sym, ir::Ty::Int(IntTy::I32));
        lowerer.insert_binding(x_sym, x_local);

        let lhs = create_ident_expr(db, "x");
        let rhs = create_int_lit(db, 5);
        let assign = create_assign_expr(db, lhs, rhs);
        let stmt = create_semi_stmt(db, assign);
        let block = create_block(db, vec![stmt]);

        let block_expr = Expr {
            id: test_node_id(),
            kind: ExprKind::Block(Box::new(block)),
            span: test_span(db),
        };

        let result = lowerer.lower_expr(&block_expr);
        assert!(result.is_ok());

        // Should have created local for x (literal is a constant)
        assert_eq!(lowerer.local_decls.len(), 1);
    }
}
