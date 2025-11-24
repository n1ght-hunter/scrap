pub mod language;
pub mod parser;
pub mod syntax_kind;

pub use language::{ScrapLanguage, SyntaxElement, SyntaxNode, SyntaxToken};
pub use syntax_kind::SyntaxKind;

use parser::Parser;
use rowan::GreenNode;
use scrap_lexer::LexedTokens;

/// Parse a file and return the green tree (CST)
#[salsa::tracked]
pub fn parse_file<'db>(
    db: &'db dyn scrap_shared::Db,
    file: scrap_shared::salsa::InputFile<'db>,
    tokens: LexedTokens<'db>,
) -> ParsedFile<'db> {
    let source = file.content(db).to_string();
    let mut parser = Parser::new(db, source, &tokens);
    parser.parse_source_file();
    let (green, errors) = parser.finish();

    ParsedFile::new(db, green, errors)
}

#[salsa::tracked(debug)]
pub struct ParsedFile<'db> {
    pub green: GreenNode,
    pub errors: Vec<String>,
}

impl<'db> ParsedFile<'db> {
    /// Get the syntax tree root
    pub fn syntax(&self, db: &'db dyn scrap_shared::Db) -> SyntaxNode {
        SyntaxNode::new_root(self.green(db).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        // TODO: Add tests once we have a way to create a test database
    }
}
