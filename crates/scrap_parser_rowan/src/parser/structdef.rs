use scrap_lexer::Token;

use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse a struct definition
    pub(super) fn parse_struct(&mut self) {
        self.start_node(SyntaxKind::STRUCT_DEF);

        self.expect(Token::Struct);

        // Struct name
        if self.at(Token::Ident) {
            self.start_node(SyntaxKind::NAME);
            self.bump();
            self.finish_node();
        }

        // Fields
        if self.at(Token::LBrace) {
            self.parse_field_list();
        }

        self.finish_node();
    }

    /// Parse struct field list
    fn parse_field_list(&mut self) {
        self.start_node(SyntaxKind::FIELD_LIST);

        self.expect(Token::LBrace);

        while !self.at(Token::RBrace) && !self.at_eof() {
            self.parse_field();

            if self.at(Token::Comma) {
                self.bump();
            } else if !self.at(Token::RBrace) {
                break;
            }
        }

        self.expect(Token::RBrace);
        self.finish_node();
    }

    /// Parse a struct field
    fn parse_field(&mut self) {
        self.start_node(SyntaxKind::FIELD);

        // Field name
        if self.at(Token::Ident) {
            self.start_node(SyntaxKind::NAME);
            self.bump();
            self.finish_node();
        }

        // Type annotation
        if self.at(Token::Colon) {
            self.start_node(SyntaxKind::TYPE_ANNOTATION);
            self.bump(); // :
            self.parse_type();
            self.finish_node();
        }

        self.finish_node();
    }
}
