use scrap_lexer::Token;

use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse a top-level item
    pub(super) fn parse_item(&mut self) {
        // Check for visibility
        let has_vis = self.at(Token::Pub);
        if has_vis {
            self.start_node(SyntaxKind::VISIBILITY);
            self.bump(); // pub
            self.finish_node();
        }

        match self.current_kind() {
            Some(Token::Fn) => self.parse_function(),
            Some(Token::Struct) => self.parse_struct(),
            Some(Token::Enum) => self.parse_enum(),
            Some(Token::Mod) => self.parse_module(),
            Some(Token::Use) => self.parse_use(),
            _ => {
                self.error(format!("Expected item, found {:?}", self.current_kind()));
            }
        }
    }

    /// Parse a use statement
    pub(super) fn parse_use(&mut self) {
        self.start_node(SyntaxKind::USE_TREE);

        self.expect(Token::Use);
        self.parse_path();
        self.expect(Token::Semicolon);

        self.finish_node();
    }

    /// Parse a module
    pub(super) fn parse_module(&mut self) {
        self.start_node(SyntaxKind::MODULE);

        self.expect(Token::Mod);

        // Module name
        if self.at(Token::Ident) {
            self.start_node(SyntaxKind::NAME);
            self.bump();
            self.finish_node();
        }

        if self.at(Token::Semicolon) {
            self.bump();
        } else if self.at(Token::LBrace) {
            // Inline module
            self.bump(); // {
            while !self.at(Token::RBrace) && !self.at_eof() {
                if self.current_kind().map_or(false, |k| k.is_trivia()) {
                    self.bump();
                    continue;
                }
                self.parse_item();
            }
            self.expect(Token::RBrace);
        }

        self.finish_node();
    }
}
