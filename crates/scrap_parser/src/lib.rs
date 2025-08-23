//! # Scrap Parser
//!
//! A parser for the Scrap programming language that generates an Abstract Syntax Tree (AST)
//! following the exact structure and patterns of the official Rust AST.
//!
//! ## Design Philosophy
//!
//! This parser is designed to be a **subset** of the Rust AST, meaning:
//! - All AST node structures exactly match their Rust counterparts
//! - Field names, types, and semantics follow Rust conventions
//! - Comments are taken directly from the official Rust documentation
//! - Only essential features are included (no macros, attributes, complex generics)
//!
//! ## Key Features
//!
//! ### Expression System
//! - **Literals**: Integers, floats, strings, booleans
//! - **Binary Operations**: Arithmetic (`+`, `-`, `*`, `/`), comparisons (`==`, `<`, etc.)
//! - **Function Calls**: Full argument parsing with proper precedence
//! - **Control Flow**: If-else expressions, block expressions
//! - **Collections**: Array literals (`[1, 2, 3]`)
//! - **Parentheses**: Precedence override with `(expr)`
//!
//! ### Statement System
//! - **Let Bindings**: Variable declarations (`let x = 5;`)
//! - **Expression Statements**: Both with and without semicolons
//! - **Item Definitions**: Functions, structs (planned)
//!
//! ### Parser Architecture
//! - **Modular Design**: Separate modules for expressions, statements, literals
//! - **Error Recovery**: Graceful handling of syntax errors
//! - **Precedence Handling**: Correct operator precedence following mathematical conventions
//! - **Chumsky Framework**: Built on the Chumsky parser combinator library
//!
//! ## AST Structure Compliance
//!
//! The AST structures in this crate are direct subsets of the Rust AST:
//!
//! ```rust,ignore
//! // Our Expr matches rustc_ast::ast::Expr exactly:
//! pub struct Expr {
//!     pub id: NodeId,      // ✓ Same as Rust
//!     pub kind: ExprKind,  // ✓ Same as Rust  
//!     pub span: Span,      // ✓ Same as Rust
//!     // attrs and tokens omitted for simplicity
//! }
//!
//! // Our ExprKind is a subset of rustc_ast::ast::ExprKind:
//! pub enum ExprKind {
//!     Array(LocalVec<Box<Expr>>),                    // ✓ ThinVec -> LocalVec
//!     Call(Box<Expr>, LocalVec<Box<Expr>>),          // ✓ Exact match
//!     Binary(BinOp, Box<Expr>, Box<Expr>),           // ✓ Exact match
//!     Lit(Lit),                                      // ✓ Exact match
//!     If(Box<Expr>, Box<Block>, Option<Box<Expr>>),  // ✓ Exact match
//!     Block(Box<Block>),                             // ✓ Simplified (no Label)
//!     Path(String),                                  // ✓ Simplified Path
//!     Paren(Box<Expr>),                              // ✓ Exact match
//!     Err,                                           // ✓ Simplified ErrorGuaranteed
//! }
//! ```
//!
//! ## Documentation Source
//!
//! All comments and documentation are taken directly from the official Rust AST documentation
//! at <https://doc.rust-lang.org/nightly/nightly-rustc/rustc_ast/ast/index.html>, ensuring
//! consistency and accuracy with the reference implementation.

use std::path::Path;

use anyhow::Context;
use ariadne::{Color, Label, Report, ReportKind};
use chumsky::{input::Stream, prelude::*};
use parser::{file_parser, item::Item};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use scrap_lexer::{Logos, Token};

pub mod ast;
pub mod parser;
pub mod utils;

pub type Span = SimpleSpan;

#[derive(Debug, Clone, Copy)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
    #[error("IO error: {0}")]
    Io(std::io::Error),
    #[error("Parse error: {0}")]
    Parse(anyhow::Error),
    #[error("Parser errors found")]
    Parser(Vec<Rich<'a, Token<'a>>>),
}

pub fn parse_files(
    files: impl IntoParallelIterator<Item = impl AsRef<Path>>,
) -> anyhow::Result<Vec<(String, Vec<Item>)>> {
    let res = files
        .into_par_iter()
        .map(|file| {
            let file = file.as_ref();
            if !file.exists() {
                anyhow::bail!("File does not exist: {}", file.display());
            }
            let content = std::fs::read_to_string(file)
                .with_context(|| format!("Failed to read file: {}", file.display()))?;
            let filename = file
                .file_name()
                .context("Failed to get file name")?
                .to_string_lossy()
                .into_owned();

            match parse_file_str(&content) {
                Ok(ast) => {
                    if ast.is_none() {
                        anyhow::bail!("No items found in file: {}", file.display());
                    }

                    Ok((filename, ast.unwrap()))
                }
                Err(parse_errs) => {
                    let source = ariadne::Source::from(&content);
                    parse_errs
                        .into_iter()
                        .map(|e| {
                            Report::build(ReportKind::Error, (&filename, e.span().into_range()))
                                .with_config(
                                    ariadne::Config::new()
                                        .with_index_type(ariadne::IndexType::Byte),
                                )
                                .with_message(e.to_string())
                                .with_label(
                                    Label::new((&filename, e.span().into_range()))
                                        .with_message(e.reason().to_string())
                                        .with_color(Color::Red),
                                )
                                .with_labels(e.contexts().map(|(label, span)| {
                                    Label::new((&filename, span.into_range()))
                                        .with_message(format!("while parsing this {label}"))
                                        .with_color(Color::Yellow)
                                }))
                                .finish()
                                .print((&filename, &source))
                        })
                        .inspect(|res| {
                            if let Err(e) = res {
                                tracing::error!("Failed to report parse errors: {}", e);
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .context("Failed to report parse errors")?;

                    anyhow::bail!("Failed to parse file: {}", file.display());
                }
            }
        })
        .inspect(|res| {
            if let Err(e) = res {
                tracing::error!("Failed to parse file: {}", e);
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(res)
}

pub fn parse_file_str<'a>(content: &'a str) -> Result<Option<Vec<Item>>, Vec<Rich<'a, Token<'a>>>> {
    let (token_iter, mut lex_errs) = scrap_lexer::Token::lexer(content).spanned().fold(
        (Vec::new(), Vec::new()),
        |(mut tokens, mut token_errors), (new_tok, new_span)| {
            let span = SimpleSpan::from(new_span);
            match new_tok {
                Ok(new_tok) => tokens.push((new_tok, span)),
                Err(e) => token_errors.push(Rich::<Token, _>::custom(span, e.to_string())),
            }
            (tokens, token_errors)
        },
    );

    let token_stream =
        Stream::from_iter(token_iter).map((0..content.len()).into(), |(t, s): (_, _)| (t, s));

    let mut state = parser::State {};

    let (ast, mut parse_errs) = file_parser()
        .parse_with_state(token_stream, &mut state)
        .into_output_errors();

    if parse_errs.is_empty() && lex_errs.is_empty() {
        return Ok(ast);
    }

    parse_errs.append(&mut lex_errs);

    Err(parse_errs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scrap_lexer::{Logos, Token};

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
