use scrap_lexer::Token;

use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse an enum definition
    pub(super) fn parse_enum(&mut self) {
        self.start_node(SyntaxKind::ENUM_DEF);

        self.expect(Token::Enum);

        // Enum name
        if self.at(Token::Ident) {
            self.start_node(SyntaxKind::NAME);
            self.bump();
            self.finish_node();
        }

        // Variants
        if self.at(Token::LBrace) {
            self.parse_variant_list();
        }

        self.finish_node();
    }

    /// Parse enum variant list
    fn parse_variant_list(&mut self) {
        self.start_node(SyntaxKind::VARIANT_LIST);

        self.expect(Token::LBrace);

        while !self.at(Token::RBrace) && !self.at_eof() {
            self.parse_variant();

            if self.at(Token::Comma) {
                self.bump();
            } else if !self.at(Token::RBrace) {
                break;
            }
        }

        self.expect(Token::RBrace);
        self.finish_node();
    }

    /// Parse an enum variant
    fn parse_variant(&mut self) {
        self.start_node(SyntaxKind::VARIANT);

        // Variant name
        if self.at(Token::Ident) {
            self.start_node(SyntaxKind::NAME);
            self.bump();
            self.finish_node();
        }

        // Optional tuple variant
        if self.at(Token::LParen) {
            self.bump(); // (
            self.parse_type();
            self.expect(Token::RParen);
        }

        self.finish_node();
    }
}
