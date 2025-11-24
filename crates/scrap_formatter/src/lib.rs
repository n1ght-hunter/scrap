mod formatter;

pub use formatter::{format_file, format_syntax_tree, FormatterConfig};

use rowan::GreenNode;
use scrap_parser_rowan::ParsedFile;

/// Format a source file using the Rowan CST
#[salsa::tracked]
pub fn format_parsed_file<'db>(
    db: &'db dyn scrap_shared::Db,
    parsed: ParsedFile<'db>,
) -> String {
    let config = FormatterConfig::default();
    format_syntax_tree(&parsed.syntax(db), &config)
}
