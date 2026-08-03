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

    #[scrap_macros::salsa_test]
    fn parse_index(db: &dyn scrap_shared::Db) {
        let source = "a[0]";
        let mut parser = parse_with(db, source);
        let expr = parser.parse_expr().unwrap_or_render();
        match expr.kind {
            ExprKind::Index(base, index) => {
                assert!(matches!(base.kind, ExprKind::Path(_)));
                assert!(matches!(
                    index.kind,
                    ExprKind::Lit(Lit {
                        kind: scrap_ast::lit::LitKind::Integer,
                        ..
                    })
                ));
            }
            _ => panic!("expected index expression"),
        }
    }

    #[scrap_macros::salsa_test]
    fn parse_index_with_expr(db: &dyn scrap_shared::Db) {
        let source = "a[i + 1]";
        let mut parser = parse_with(db, source);
        let expr = parser.parse_expr().unwrap_or_render();
        match expr.kind {
            ExprKind::Index(_, index) => {
                assert!(matches!(index.kind, ExprKind::Binary(..)));
            }
            _ => panic!("expected index expression"),
        }
    }

    #[scrap_macros::salsa_test]
    fn parse_chained_index(db: &dyn scrap_shared::Db) {
        let source = "a[i][j]";
        let mut parser = parse_with(db, source);
        let expr = parser.parse_expr().unwrap_or_render();
        match expr.kind {
            ExprKind::Index(base, _) => {
                assert!(matches!(base.kind, ExprKind::Index(..)));
            }
            _ => panic!("expected outer index expression"),
        }
    }

    #[scrap_macros::salsa_test]
    fn parse_index_then_field(db: &dyn scrap_shared::Db) {
        let source = "a[i].f";
        let mut parser = parse_with(db, source);
        let expr = parser.parse_expr().unwrap_or_render();
        match expr.kind {
            ExprKind::Field(base, _) => {
                assert!(matches!(base.kind, ExprKind::Index(..)));
            }
            _ => panic!("expected field access on an index expression"),
        }
    }

    #[scrap_macros::salsa_test]
    fn parse_nested_index(db: &dyn scrap_shared::Db) {
        let source = "a[b[c]]";
        let mut parser = parse_with(db, source);
        let expr = parser.parse_expr().unwrap_or_render();
        match expr.kind {
            ExprKind::Index(_, index) => {
                assert!(matches!(index.kind, ExprKind::Index(..)));
            }
            _ => panic!("expected index expression"),
        }
    }

    #[scrap_macros::salsa_test]
    fn parse_index_missing_bracket_errors(db: &dyn scrap_shared::Db) {
        let source = "a[0";
        let mut parser = parse_with(db, source);
        assert!(parser.parse_expr().is_err());
    }
}
