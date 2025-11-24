use scrap_lexer::Token;

use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse an atomic expression
    pub(in crate::parser) fn parse_atom_expr(&mut self) {
        match self.current_kind() {
            Some(Token::Int) | Some(Token::Float) | Some(Token::Str) | Some(Token::Bool) => {
                self.start_node(SyntaxKind::LITERAL_EXPR);
                self.bump();
                self.finish_node();
            }
            Some(Token::Ident) => {
                // Lookahead to see if this is a function call
                let is_call = self.nth_at(1, Token::LParen);

                if is_call {
                    // Parse as CALL_EXPR containing PATH_EXPR
                    self.start_node(SyntaxKind::CALL_EXPR);
                    self.start_node(SyntaxKind::PATH_EXPR);
                    self.parse_path();
                    self.finish_node(); // PATH_EXPR
                    self.parse_arg_list();
                    self.finish_node(); // CALL_EXPR
                } else {
                    // Just a PATH_EXPR
                    self.start_node(SyntaxKind::PATH_EXPR);
                    self.parse_path();
                    self.finish_node();
                }
            }
            Some(Token::LParen) => {
                self.start_node(SyntaxKind::PAREN_EXPR);
                self.bump(); // (
                self.parse_expr();
                self.expect(Token::RParen);
                self.finish_node();
            }
            Some(Token::LBracket) => {
                self.start_node(SyntaxKind::ARRAY_EXPR);
                self.bump(); // [
                while !self.at(Token::RBracket) && !self.at_eof() {
                    self.parse_expr();
                    if self.at(Token::Comma) {
                        self.bump();
                    } else {
                        break;
                    }
                }
                self.expect(Token::RBracket);
                self.finish_node();
            }
            Some(Token::If) => self.parse_if_expr(),
            Some(Token::Return) => {
                self.start_node(SyntaxKind::RETURN_EXPR);
                self.bump(); // return
                if !self.at(Token::Semicolon) && !self.at_eof() {
                    self.parse_expr();
                }
                self.finish_node();
            }
            Some(Token::LBrace) => self.parse_block_expr(),
            _ => {
                self.error(format!(
                    "Expected expression, found {:?}",
                    self.current_kind()
                ));
            }
        }
    }
}
