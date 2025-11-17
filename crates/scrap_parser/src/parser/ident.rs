use scrap_ast::ident::Ident;
use scrap_lexer::Token;
use scrap_span::Symbol;

impl<'a, 'db> super::Parser<'a, 'db> {
    /// Check if the current token is an identifier
    pub fn check_ident(&mut self) -> bool {
        self.check(Token::Ident)
    }

    pub fn parse_ident(&mut self) -> crate::PResult<'a, Ident<'db>> {
        let span = self.token.span;
        if self.eat(Token::Ident) {
            let name = &self.source[span.to_range(self.db)];
            let key = Symbol::new(self.db, name);
            let id = self.state.new_node_id();
            Ok(Ident {
                id,
                name: key,
                span,
            })
        } else {
            Err(self.unexpected_token_error(&[Token::Ident]))
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::parse_test_utils::parse_with;

    #[test]
    fn test_parse_ident() {
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = parse_with(&db, "my_variable");
        let ident = match parser.parse_ident() {
            Ok(ident) => ident,
            Err(e) => {
                panic!("Failed to parse ident: {:?}", e);
            }
        };
        assert_eq!(ident.name.text(&db), "my_variable");
        assert_eq!(ident.span.to_range(&db), 0..11);
    }
}
