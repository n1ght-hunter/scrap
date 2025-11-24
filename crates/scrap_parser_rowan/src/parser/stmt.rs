use scrap_lexer::Token;

use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse a statement
    pub(super) fn parse_stmt(&mut self) {
        if self.at(Token::Let) {
            self.parse_let_stmt();
        } else {
            // Expression statement
            self.start_node(SyntaxKind::EXPR_STMT);
            self.parse_expr();
            if self.at(Token::Semicolon) {
                self.bump();
            }
            self.finish_node();
        }
    }

    /// Parse a let statement
    fn parse_let_stmt(&mut self) {
        self.start_node(SyntaxKind::LET_STMT);

        self.expect(Token::Let);

        // Pattern (for now just identifier)
        if self.at(Token::Ident) {
            self.start_node(SyntaxKind::IDENT_PAT);
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

        // Initializer
        if self.at(Token::Assign) {
            self.bump(); // =
            self.parse_expr();
        }

        self.expect(Token::Semicolon);
        self.finish_node();
    }
}
