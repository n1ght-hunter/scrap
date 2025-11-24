use scrap_lexer::Token;

use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse a function definition
    pub(super) fn parse_function(&mut self) {
        self.start_node(SyntaxKind::FUNCTION);

        self.expect(Token::Fn);

        // Function name
        if self.at(Token::Ident) {
            self.start_node(SyntaxKind::NAME);
            self.bump();
            self.finish_node();
        } else {
            self.error("Expected function name".to_string());
        }

        // Parameters
        self.parse_param_list();

        // Return type
        if self.at(Token::Arrow) {
            self.start_node(SyntaxKind::RET_TYPE);
            self.bump(); // ->
            self.parse_type();
            self.finish_node();
        }

        // Body
        if self.at(Token::LBrace) {
            self.parse_block_expr();
        } else {
            self.error("Expected function body".to_string());
        }

        self.finish_node();
    }

    /// Parse parameter list
    fn parse_param_list(&mut self) {
        self.start_node(SyntaxKind::PARAM_LIST);

        self.expect(Token::LParen);

        while !self.at(Token::RParen) && !self.at_eof() {
            // Skip trivia
            if self.current_kind().map_or(false, |k| k.is_trivia()) {
                self.bump();
                continue;
            }

            self.parse_param();

            if self.at(Token::Comma) {
                self.bump();
            } else if !self.at(Token::RParen) {
                break;
            }
        }

        self.expect(Token::RParen);
        self.finish_node();
    }

    /// Parse a single parameter
    fn parse_param(&mut self) {
        self.start_node(SyntaxKind::PARAM);

        // Parameter name
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
