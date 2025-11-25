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
    use expect_test::expect;

    #[test]
    fn test_format_enum() {
        let source = r#"enum MyEnum {
    Variant1,
    Variant2(MyStruct),
}"#;
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        expect![[r#"
            enum MyEnum {
                Variant1,
                Variant2(MyStruct),
            }
        "#]]
        .assert_eq(&formatted);
    }

    #[test]
    fn test_format_struct() {
        let source = r#"struct MyStruct {
    field1: i32,
    field2: String,
}"#;
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        expect![[r#"
            struct MyStruct {
                field1: i32,
                field2: String,
            }
        "#]]
        .assert_eq(&formatted);
    }

    #[test]
    fn test_format_function() {
        let source = r#"fn main() {
    print("Hello, world!");
}"#;
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        expect![[r#"
            fn main() {
                print("Hello, world!");
            }
        "#]]
        .assert_eq(&formatted);
    }

    #[test]
    fn test_format_binary_expr_in_block() {
        let source = "fn main() { a+b }";
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        expect![[r#"
            fn main() {
                a + b
            }
        "#]]
        .assert_eq(&formatted);
    }

    #[test]
    fn test_format_if_with_binary_expr_block() {
        let source = "fn foo() { if a>1.0 { a+b } else { 50.0 } }";
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        expect![[r#"
            fn foo() {
                if a > 1.0 {
                    a + b
                } else {
                    50.0
                }
            }
        "#]]
        .assert_eq(&formatted);
    }

    #[test]
    fn test_format_chained_binary_expr() {
        let source = "fn main() { a+b+c }";
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        expect![[r#"
            fn main() {
                a + b + c
            }
        "#]]
        .assert_eq(&formatted);
    }

    #[test]
    fn test_format_complex_precedence() {
        let source = "fn main() { a*b+c*d }";
        let config = FormatterConfig::default();
        let formatted = format_file(source, &config);

        expect![[r#"
            fn main() {
                a * b + c * d
            }
        "#]]
        .assert_eq(&formatted);
    }
}
