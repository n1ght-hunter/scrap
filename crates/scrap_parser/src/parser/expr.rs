use scrap_ast::{
    block::Block,
    expr::{Expr, ExprKind},
};
use scrap_lexer::Token;
use scrap_span::{Span, Spanned};

impl<'a> super::NewParser<'a> {
    pub fn parse_expr(&mut self) -> crate::PResult<'a, Expr> {
        let start = self.token.span.start;

        let kind = self.parse_expr_kind()?;

        Ok(Expr {
            id: self.state.new_node_id(),
            kind: kind,
            span: Span::new(start..self.token.span.end),
        })
    }

    pub fn parse_expr_kind(&mut self) -> crate::PResult<'a, ExprKind> {
        if self.check(Token::LBrace) {
            let block = self.parse_block()?;
            return Ok(ExprKind::Block(Box::new(block)));
        }
        if self.check(Token::LBracket) {
            unimplemented!("array expression parsing not implemented yet");
        }
        if self.check(Token::Ident) {
            if let Some(Spanned {
                node: Token::LParen,
                ..
            }) = self.look_ahead(1)
            {
                unimplemented!("function call expression parsing not implemented yet");
            }
        }

        if self.eat(Token::Return) {
            let expr = if !self.eat(Token::Semicolon) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            return Ok(ExprKind::Return(expr.map(Box::new)));
        }

        Err(self.unexpected_token_error(&[
            Token::LBrace,
            Token::LBracket,
            Token::Ident,
            Token::Return,
        ]))
    }
}

#[cfg(test)]
mod tests {
    use scrap_ast::expr::ExprKind;

    use crate::parser::parse_test_utils::{ExtendRes, parse_with};


    #[test]
    fn parse_return() {
         let source = "return;";
        let mut parser = parse_with(source);
        let expr = parser.parse_expr().unwrap_or_render();
        assert!(matches!(expr.kind, ExprKind::Return(None)));
    }

    #[test]
    fn parse_return_with_expr() {
         let source = "return 42;";
        let mut parser = parse_with(source);
        let expr = parser.parse_expr().unwrap_or_render();
        match expr.kind {
            ExprKind::Return(Some(ret_expr)) => {
                assert!(matches!(ret_expr.kind, ExprKind::Lit(_)));
            }
            _ => panic!("expected return expression with value"),
        }
    }
}
