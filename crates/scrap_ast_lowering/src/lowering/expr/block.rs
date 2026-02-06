//! Block lowering

use scrap_ast::block::Block;
use scrap_ir as ir;

use crate::{lowerer::ExprLowerer, MResult};

impl<'db> ExprLowerer<'db> {
    /// Lower a block expression
    pub(crate) fn lower_block_expr(&mut self, block: &Block<'db>) -> MResult<ir::Operand<'db>> {
        self.lower_block(block)?;
        // Blocks produce void for now
        // TODO: Handle implicit return from last expression
        Ok(ir::Operand::Constant(ir::Constant::Void))
    }

    /// Lower a block's statements
    pub(crate) fn lower_block(&mut self, block: &Block<'db>) -> MResult<()> {
        for stmt in &block.stmts {
            match &stmt.kind {
                scrap_ast::stmt::StmtKind::Semi(expr) | scrap_ast::stmt::StmtKind::Expr(expr) => {
                    // Lower the expression for side effects
                    self.lower_expr(expr)?;
                }
                scrap_ast::stmt::StmtKind::Let(_local) => {
                    // TODO: Handle let statements with initializers
                    // For now, skip
                }
                scrap_ast::stmt::StmtKind::Item(_) | scrap_ast::stmt::StmtKind::Empty => {
                    // Skip these for now
                }
            }
        }
        Ok(())
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

        // Should have created locals for x and the literal
        assert_eq!(lowerer.local_decls.len(), 2);
    }
}
