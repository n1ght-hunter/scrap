use scrap_ast::local::{Local, LocalKind};
use scrap_lexer::Token;
use scrap_span::Span;

impl<'a, 'db> super::Parser<'a, 'db> {
    /// let [mut] <pat>:<ty> = <expr>;
    pub fn parse_local(&mut self) -> crate::PResult<'a, Local<'db>> {
        let start = self.token.span.start(self.db);
        self.expect(Token::Let)?;

        // Check for `mut` keyword (contextual — parsed as identifier)
        let mutability = if self.check(Token::Ident) {
            let text = &self.source[self.token.span.to_range(self.db)];
            if text == "mut" {
                self.bump();
                scrap_shared::Mutability::Mut
            } else {
                scrap_shared::Mutability::Not
            }
        } else {
            scrap_shared::Mutability::Not
        };

        let pat = self.parse_pat_with_mutability(mutability)?;

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
