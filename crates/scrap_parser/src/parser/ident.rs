use scrap_ast::ident::Ident;
use scrap_diagnostics::{Level, annotate_snippets::Group};
use scrap_lexer::Token;

impl<'a> super::NewParser<'a> {
    /// Check if the current token is an identifier
    pub fn check_ident(&mut self) -> bool {
        self.check(Token::Ident)
    }

    pub fn parse_ident(&mut self) -> crate::PResult<'a, Ident> {
        let span = self.token.span;
        if self.eat(Token::Ident) {
            let name = &self.source[span.to_range()];
            let key = self.get_or_intern(name);
            let id = self.state.new_node_id();
            Ok(Ident { id, name: key, span })
        } else {
            Err(Group::with_title(Level::ERROR.primary_title(format!(
                "Expected identifier found `{}`",
                self.token.node
            ))))
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::parse_test_utils::parse_with;

    use super::*;

    #[test]
    fn test_parse_ident() {
        let mut parser = parse_with("my_variable");
        let ident = match parser.parse_ident() {
            Ok(ident) => ident,
            Err(e) => {
                panic!("Failed to parse ident: {:?}", e);
            }
        };
        assert_eq!(parser.resolve_symbol(ident.name), "my_variable");
        assert_eq!(ident.span.to_range(), 0..11);
    }
}
