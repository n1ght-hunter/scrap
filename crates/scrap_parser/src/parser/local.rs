use scrap_ast::local::{Local, LocalKind};
use scrap_lexer::Token;
use scrap_span::Span;

impl<'a, 'db> super::Parser<'a, 'db> {
    /// let <pat>:<ty> = <expr>;
    pub fn parse_local(&mut self) -> crate::PResult<'a, Local<'db>> {
        let start = self.token.span.start(self.db);
        self.expect(Token::Let)?;
        let pat = self.parse_pat()?;

        let mut ty = None;
        if self.eat(Token::Colon) {
            ty = Some(self.parse_type()?);
        }

        if self.eat(Token::Semicolon) {
            return Ok(Local {
                id: self.state.new_node_id(),
                span: Span::new(self.db, start, self.token.span.end(self.db)),
                pat: Box::new(pat),
                ty: None,
                kind: LocalKind::Decl,
            });
        }

        self.expect(Token::Assign)?;

        let expr = self.parse_expr()?;

        self.expect(Token::Semicolon)?;

        Ok(Local {
            id: self.state.new_node_id(),
            span: Span::new(self.db, start, expr.span.end(self.db)),
            pat: Box::new(pat),
            ty,
            kind: LocalKind::Init(Box::new(expr)),
        })
    }
}
