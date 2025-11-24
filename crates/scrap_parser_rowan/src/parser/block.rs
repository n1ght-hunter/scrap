use scrap_lexer::Token;

use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse a block expression
    pub(super) fn parse_block_expr(&mut self) {
        self.start_node(SyntaxKind::BLOCK_EXPR);
        self.start_node(SyntaxKind::BLOCK);

        self.expect(Token::LBrace);

        self.start_node(SyntaxKind::STMT_LIST);
        while !self.at(Token::RBrace) && !self.at_eof() {
            self.parse_stmt();
        }
        self.finish_node(); // STMT_LIST

        self.expect(Token::RBrace);
        self.finish_node(); // BLOCK
        self.finish_node(); // BLOCK_EXPR
    }
}
