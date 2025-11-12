//! # Scrap Parser
//!
//! A parser for the Scrap programming language that generates an Abstract Syntax Tree (AST)
//! following the exact structure and patterns of the official Rust AST.
#![feature(
    allocator_api,
    try_blocks,
    gen_blocks,
    default_field_values,
    negative_impls
)]

use std::sync::Arc;

use scrap_diagnostics::annotate_snippets::Group;
use scrap_lexer::Token;
use scrap_span::Spanned;

pub mod parser;
mod errors;

pub type PResult<'a, T> = std::result::Result<T, Group<'a>>;
pub type TokenStream<'db> = Arc<Vec<Spanned<'db, Token>>>;


#[cfg(target_arch = "wasm32")]
mod tests {
    use std::path::PathBuf;

    use super::*;

    const TEST_AST: &str = r#"
    fn foo(a: f64, b: f64) -> f64 {
        let c = if a > 10.0 {
            a + b
        } else {
            50.0
        };
        c + 2.0
    }

    fn bar() -> String {
        "Hello, \\world!"
    }

    enum MyEnum {
        Variant1,
        Variant2(MyStruct),
    }

    struct MyStruct {
        field1: i32,
        field2: String,
    }
    "#;

    #[test]
    fn parse_basic_function() -> anyhow::Result<()> {
        let filename = "basic.sc";
        parse_files([format!("../../example/{}", filename)])?;
        Ok(())
    }

    #[test]
    fn allfiles() -> anyhow::Result<()> {
        let dir = PathBuf::from("../../tests");
        let mut files = Vec::new();
        for entry in dir.read_dir()? {
            let entry = entry?;
            if entry.path().is_file() {
                files.push(entry.path());
            }
        }
        let _adsf = parse_files(files)?;

        Ok(())
    }

    #[test]
    fn test_ast() {
        let src = TEST_AST;
        parse_file_str(&src).unwrap();
    }

    #[test]
    fn test_return_statements() {
        let src = r#"
        fn test_return() -> i32 {
            return 42;
        }

        fn test_early_return(x: i32) -> i32 {
            if x > 10 {
                return x * 2;
            }
            return x + 1;
        }

        fn test_no_return_type() {
            return;
        }

        fn test_optional_return() {
            let x = 5;
        }
        "#;
        parse_file_str(&src).unwrap();
    }

    #[test]
    fn test_simple_return_functionality() -> anyhow::Result<()> {
        let filename = "simple_return_test.sc";
        let src = std::fs::read_to_string(format!("../../example/{}", filename))?;
        parse_file_str(&src).unwrap();
        Ok(())
    }

    #[test]
    fn test_return_requires_semicolon() {
        let src = r#"
        fn test_return_no_semicolon() -> i32 {
            return 42
        }
        "#;

        parse_file_str(&src).unwrap_err();
    }

    #[test]
    fn test_return_with_semicolon_works() {
        let src = r#"
        fn test_return_with_semicolon() -> i32 {
            return 42;
        }
        
        fn test_void_return() {
            return;
        }
        "#;

        parse_file_str(&src).unwrap();
    }

    #[test]
    fn test_missing_semicolon_error_quality() {
        let src = r#"
        fn foo(a: f64, b: f64) -> f64 {
            let c = if a > 1.0 {
                a + b
            } else {
                50.0
            }  // Missing semicolon here - should give helpful error
            c + 2.0
        }
        "#;

        parse_file_str(&src).unwrap_err();
    }

    #[test]
    fn test_various_missing_semicolon_cases() {
        // Test case 1: Missing semicolon after simple let statement
        let test1 = r#"
        fn test() {
            let x = 5  // Missing semicolon
            let y = 10;
        }
        "#;

        let result1 = parse_file_str(&test1);
        assert!(
            result1.is_err(),
            "Expected error for missing semicolon after simple let"
        );

        // Test case 2: Missing semicolon after function call
        let test2 = r#"
        fn test() {
            foo()  // Missing semicolon
            bar();
        }
        "#;

        let result2 = parse_file_str(&test2);
        assert!(
            result2.is_err(),
            "Expected error for missing semicolon after function call"
        );

        // Test case 3: Missing semicolon after return statement
        let test3 = r#"
        fn test() -> i32 {
            return 42  // Missing semicolon
        }
        "#;

        let result3 = parse_file_str(&test3);
        assert!(
            result3.is_err(),
            "Expected error for missing semicolon after return"
        );
    }

    #[test]
    fn test_correct_semicolon_usage() {
        // These should all parse successfully
        let correct_cases = vec![
            r#"
            fn test() -> i32 {
                let x = 5;
                return x;
            }
            "#,
            r#"
            fn test() -> i32 {
                let x = if true { 1 } else { 2 };
                x + 1
            }
            "#,
            r#"
            fn test() {
                let x = 5;
                foo();
                bar();
            }
            "#,
        ];

        for (i, test_case) in correct_cases.iter().enumerate() {
            let result = parse_file_str(test_case);
            assert!(
                result.is_ok(),
                "Expected successful parse for correct case {}",
                i
            );
        }
    }
}
