mod formatter;

pub use formatter::{FormatterConfig, format_file, format_syntax_tree};

use scrap_parser_rowan::ParsedFile;

/// Format a source file using the Rowan CST
#[salsa::tracked]
pub fn format_parsed_file<'db>(db: &'db dyn scrap_shared::Db, parsed: ParsedFile<'db>) -> String {
    let config = FormatterConfig::default();
    format_syntax_tree(&parsed.syntax(db), &config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_enum() {
        let source = r#"enum MyEnum {
    Variant1,
    Variant2(MyStruct),
}"#;
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        eprintln!("=== FORMATTED OUTPUT ===\n{}\n===", formatted);

        // Just verify it doesn't crash
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_format_struct() {
        let source = r#"struct MyStruct {
    field1: i32,
    field2: String,
}"#;
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        eprintln!("=== FORMATTED OUTPUT ===\n{}\n===", formatted);

        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_format_function() {
        let source = r#"fn main() {
    print("Hello, world!");
}"#;
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        eprintln!("=== FORMATTED OUTPUT ===\n{}\n===", formatted);

        assert!(!formatted.is_empty());
    }
}
