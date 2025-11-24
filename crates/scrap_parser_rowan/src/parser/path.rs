use scrap_lexer::Token;

use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse a path
    pub(super) fn parse_path(&mut self) {
        self.start_node(SyntaxKind::PATH);

        loop {
            if self.at(Token::Ident) {
                self.start_node(SyntaxKind::PATH_SEGMENT);
                self.bump();
                self.finish_node();
            } else {
                break;
            }

            if self.at(Token::DoubleColon) {
                self.bump();
            } else {
                break;
            }
        }

        self.finish_node();
    }
}
