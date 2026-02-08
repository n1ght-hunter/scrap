use crate::PResult;
use scrap_ast::typedef::{Ty, TyKind};
use scrap_lexer::Token;
use scrap_shared::path::Path;
use scrap_shared::types::Mutability;
use scrap_span::Span;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_type(&mut self) -> PResult<'a, Ty<'db>> {
        // Never type: `!`
        if self.check(Token::Bang) {
            let span = self.token.span;
            self.bump();
            return Ok(Ty {
                id: self.state.new_node_id(),
                span,
                kind: TyKind::Never,
            });
        }

        // Pointer type: `*T`
        if self.check(Token::Mul) {
            let start_span = self.token.span;
            self.bump(); // consume `*`
            let inner = self.parse_type()?;
            let end_span = inner.span;
            return Ok(Ty {
                id: self.state.new_node_id(),
                span: Span::new(self.db, start_span.start(self.db), end_span.end(self.db)),
                kind: TyKind::Ptr(Box::new(inner)),
            });
        }

        // Reference types: `&T` or `&mut T`
        if self.check(Token::BitAnd) {
            let start_span = self.token.span;
            self.bump(); // consume `&`

            // Check for `mut` keyword (it's an Ident, not a keyword)
            let mutability = if self.check(Token::Ident) {
                let text = &self.source[self.token.span.to_range(self.db)];
                if text == "mut" {
                    self.bump(); // consume `mut`
                    Mutability::Mut
                } else {
                    Mutability::Not
                }
            } else {
                Mutability::Not
            };

            let inner = self.parse_type()?;
            let end_span = inner.span;
            return Ok(Ty {
                id: self.state.new_node_id(),
                span: Span::new(self.db, start_span.start(self.db), end_span.end(self.db)),
                kind: TyKind::Ref(Box::new(inner), mutability),
            });
        }

        // Identifier types (i32, bool, MyStruct, etc.)
        let ident = self.parse_ident()?;
        Ok(Ty {
            id: self.state.new_node_id(),
            span: ident.span,
            kind: TyKind::Path(Path::from_ident(ident)),
        })
    }
}
