use crate::PResult;
use scrap_ast::generics::{GenericParam, GenericParamKind, Generics};
use scrap_lexer::Token;
use scrap_span::Span;
use thin_vec::ThinVec;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn parse_generics(&mut self) -> PResult<'a, Generics<'db>> {
        if !self.eat(Token::Lt) {
            return Ok(Generics::default());
        }

        let mut params = ThinVec::new();
        while !self.check(Token::Gt) {
            let param_start = self.token.span;
            let ident = self.parse_ident()?;
            let span = Span::new(self.db, param_start.start(self.db), ident.span.end(self.db));

            params.push(GenericParam {
                id: self.state.new_node_id(),
                ident,
                kind: GenericParamKind::Type,
                bounds: ThinVec::new(),
                span,
            });

            if !self.eat(Token::Comma) {
                break;
            }
        }

        self.expect(Token::Gt)?;
        Ok(Generics { params })
    }
}
