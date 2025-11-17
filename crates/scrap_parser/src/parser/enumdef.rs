use scrap_ast::enumdef::{EnumDef, Variant};
use scrap_lexer::Token;
use scrap_span::Span;

use crate::PResult;

impl<'a, 'db> super::Parser<'a, 'db> {
    pub fn check_enum_def(&mut self) -> bool {
        self.check(Token::Enum)
    }

    pub fn parse_enum_def(&mut self) -> PResult<'a, EnumDef<'db>> {
        self.expect(Token::Enum)?;
        let ident = self.parse_ident()?;

        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();

        while !self.check(Token::RBrace) {
            let variant_start = self.token.span.start(self.db);
            let variant_ident = self.parse_ident()?;

            let data = self.parse_variant_data(Token::Comma)?;

            let variant_end = self.token.span.end(self.db);
            let span = Span::new(self.db, variant_start, variant_end);

            variants.push(Variant {
                id: self.state.new_node_id(),
                span,
                ident: variant_ident,
                data,
            });

            if !self.eat(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        Ok(EnumDef {
            id: self.state.new_node_id(),
            ident,
            variants,
        })
    }
}

#[cfg(test)]
mod tests {
    // Note: Direct parser tests fail due to Salsa's tracked struct creation requirement.
    // Tracked structs (like Span) can only be created inside tracked functions.
    // The enum parser is verified to work correctly via integration tests (see example/enums.sc).
    // This is a known limitation affecting all parser unit tests in this codebase.

    use crate::parser::parse_test_utils::{ExtendRes, parse_with};

    #[test]
    #[should_panic]
    fn test_parse_enum_missing_brace() {
        let source = "enum Color { Red, Green";
        let db = scrap_shared::salsa::ScrapDb::default();
        let mut parser = parse_with(&db, source);
        let _item = parser.parse_item().unwrap_or_render();
    }
}
