use scrap_ast::{
    expr::{Expr, ExprKind},
    path::{Path, PathSegment},
};
use scrap_lexer::Token;
use scrap_span::{Span, Spanned};

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_expr(&mut self) -> crate::PResult<'a, Expr<'db>> {
        let start_pos = self.token.span.start(self.db);

        let kind = self.parse_expr_kind()?;
        let end_pos = self.token.span.end(self.db);
        let span = Span::new(self.db, start_pos, end_pos);

        Ok(Expr {
            id: self.state.new_node_id(),
            kind,
            span,
        })
    }

    pub fn parse_expr_kind(&mut self) -> crate::PResult<'a, ExprKind<'db>> {
        if self.check(Token::LBrace) {
            let block = self.parse_block()?;
            return Ok(ExprKind::Block(Box::new(block)));
        }
        if self.check(Token::LBracket) {
            unimplemented!("array expression parsing not implemented yet");
        }
        if self.check_path(Token::LParen) {
            return self.function_call_expr();
        }

        if self.eat(Token::Return) {
            let expr = if !self.eat(Token::Semicolon) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            return Ok(ExprKind::Return(expr.map(Box::new)));
        }

        if self.token.node.is_literal() {
            if self.check(Token::Ident) {
                let path = self.parse_path(Token::Eof)?;
                return Ok(ExprKind::Path(path));
            } else {
                let lit = self.parse_lit()?;
                return Ok(ExprKind::Lit(lit));
            }
        }

        if self.check(Token::If) {
            return self.parse_if_expr();
        }

        Err(self.raise_unexpected_token_error())
    }

    fn function_call_expr(&mut self) -> crate::PResult<'a, ExprKind<'db>> {
        let path = self.parse_path(Token::LParen)?;
        self.expect(Token::LParen)?;
        let mut args = thin_vec::ThinVec::new();
        while !self.check(Token::RParen) {
            let arg_expr = self.parse_expr()?;
            args.push(Box::new(arg_expr));
            if !self.eat(Token::Comma) {
                break;
            }
        }
        self.expect(Token::RParen)?;
        Ok(ExprKind::Call(
            Box::new(Expr {
                id: self.state.new_node_id(),
                span: path.span,
                kind: ExprKind::Path(path),
            }),
            args,
        ))
    }

    pub fn check_path(&mut self, term: Token) -> bool {
        if self.check(Token::Ident)
            && (self.check_ahead(1, Token::DoubleColon) || self.check_ahead(1, term))
        {
            return true;
        }
        false
    }

    pub fn parse_path(&mut self, term: Token) -> crate::PResult<'a, Path<'db>> {
        let mut segments = thin_vec::ThinVec::new();

        while !self.check(term) {
            let ident = self.parse_ident()?;
            segments.push(PathSegment {
                id: self.state.new_node_id(),
                ident,
            });

            if !self.eat(Token::DoubleColon) || self.check(term) {
                break;
            }
        }

        if segments.is_empty() {
            return Err(self.unexpected_token_error(&[Token::Ident]));
        }

        Ok(Path {
            span: Span::new(
                self.db,
                segments.first().unwrap().ident.span.start(self.db),
                segments.last().unwrap().ident.span.end(self.db),
            ),
            segments,
        })
    }

    fn parse_if_expr(&mut self) -> crate::PResult<'a, ExprKind<'db>> {
        self.expect(Token::If)?;
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_span_start = self.token.span.start(self.db);
        let else_block = if self.eat(Token::Else) {
            Some(Box::new(Expr {
                id: self.state.new_node_id(),
                kind: ExprKind::Block(Box::new(self.parse_block()?)),
                span: Span::new(
                    self.db,
                    else_span_start,
                    self.token.span.end(self.db),
                ),
            }))
        } else {
            None
        };

        Ok(ExprKind::If(
            Box::new(condition),
            Box::new(then_block),
            else_block,
        ))
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
