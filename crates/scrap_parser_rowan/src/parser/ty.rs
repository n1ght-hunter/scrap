use crate::{parser::Parser, syntax_kind::SyntaxKind};

impl<'db> Parser<'db> {
    /// Parse a type
    pub(super) fn parse_type(&mut self) {
        self.start_node(SyntaxKind::PATH_TYPE);
        self.parse_path();
        self.finish_node();
    }
}
