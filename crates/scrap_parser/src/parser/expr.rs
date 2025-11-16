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

    #[test]
    fn parse_return() {
        let source = "return;";
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = parse_with(&db, source);
        let expr = parser.parse_expr().unwrap_or_render();
        assert!(matches!(expr.kind, ExprKind::Return(None)));
    }

    #[test]
    fn parse_return_with_expr() {
        let source = "return 42;";
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = parse_with(&db, source);
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
