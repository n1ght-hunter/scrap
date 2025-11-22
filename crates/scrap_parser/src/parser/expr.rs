//! Expression parsing using Pratt parser (precedence climbing) algorithm.

mod atom;
mod pratt;

use scrap_ast::expr::Expr;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_expr(&mut self) -> crate::PResult<'a, Expr<'db>> {
        self.parse_expr_with_min_precedence(0)
    }
}

#[cfg(test)]
mod tests {
    use scrap_ast::{expr::ExprKind, lit::Lit};

    use crate::parser::parse_test_utils::{ExtendRes, parse_with};

    #[scrap_macros::salsa_test]
    fn parse_return(db: &dyn scrap_shared::Db) {
        let source = "return;";
        let mut parser = parse_with(db, source);
        let expr = parser.parse_expr().unwrap_or_render();
        assert!(matches!(expr.kind, ExprKind::Return(None)));
    }

    #[scrap_macros::salsa_test]
    fn parse_return_with_expr(db: &dyn scrap_shared::Db) {
        let source = "return 42;";
        let mut parser = parse_with(db, source);
        let expr = parser.parse_expr().unwrap_or_render();
        match expr.kind {
            ExprKind::Return(Some(ret_expr)) => {
                assert!(matches!(
                    ret_expr.kind,
                    ExprKind::Lit(Lit {
                        kind: scrap_ast::lit::LitKind::Integer,
                        ..
                    })
                ));
            }
            _ => panic!("expected return expression with value"),
        }
    }
}
