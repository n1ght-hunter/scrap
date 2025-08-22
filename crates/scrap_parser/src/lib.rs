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

use chumsky::span::SimpleSpan;

pub mod ast;
pub mod parser;
pub mod utils;
pub mod error;

pub type Span = SimpleSpan;

#[derive(Debug, Clone, Copy)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use chumsky::{input::Stream, prelude::*};
    use scrap_lexer::{Logos, Token};

    use crate::parser::file_parser;

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
    fn test_unique_node_ids() -> anyhow::Result<()> {
        use crate::parser::expr::inline_expr_parser;
        use std::collections::HashSet;

        let test_expressions = vec![
            "42",    // Simple literal
            "foo",   // Simple identifier
            "x + 1", // Simple binary expression
            "a * 3", // Another binary expression
            "b + 5", // Addition
            "123",   // Another literal
            "hello", // Another identifier
        ];

        let mut all_node_ids = HashSet::new();
        let mut total_nodes = 0;

        for expr_src in test_expressions {
            println!("Testing expression: {}", expr_src);

            let (token_iter, _) = scrap_lexer::Token::lexer(expr_src).spanned().fold(
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

            let token_stream = Stream::from_iter(token_iter)
                .map((0..expr_src.len()).into(), |(t, s): (_, _)| (t, s));

            match inline_expr_parser().parse(token_stream).into_result() {
                Ok(expr) => {
                    println!("  Successfully parsed: {:?}", expr);
                    let node_ids = collect_node_ids(&expr);
                    println!("  Found {} nodes with IDs: {:?}", node_ids.len(), node_ids);

                    total_nodes += node_ids.len();

                    // Check that all IDs in this expression are unique
                    let unique_ids: HashSet<_> = node_ids.iter().cloned().collect();
                    assert_eq!(
                        unique_ids.len(),
                        node_ids.len(),
                        "Duplicate NodeIds found within expression: {}",
                        expr_src
                    );

                    // Add to global set to check across expressions
                    for id in node_ids {
                        assert!(
                            all_node_ids.insert(id),
                            "NodeId {:?} was reused across different expressions!",
                            id
                        );
                    }
                }
                Err(parse_errors) => {
                    println!("  Parse failed: {:?}", parse_errors);
                }
            }
        }

        println!(
            "Total nodes created: {}, All unique: {}",
            total_nodes,
            all_node_ids.len()
        );
        assert_eq!(
            total_nodes,
            all_node_ids.len(),
            "Some NodeIds were duplicated!"
        );

        Ok(())
    }

    /// Helper function to collect all NodeIds from an expression recursively
    fn collect_node_ids(expr: &crate::parser::expr::Expr) -> Vec<crate::ast::NodeId> {
        use crate::parser::expr::ExprKind;

        let mut ids = vec![expr.id];

        match &expr.kind {
            ExprKind::Array(exprs) => {
                for expr in exprs.iter() {
                    ids.extend(collect_node_ids(expr));
                }
            }
            ExprKind::Call(func, args) => {
                ids.extend(collect_node_ids(func));
                for arg in args.iter() {
                    ids.extend(collect_node_ids(arg));
                }
            }
            ExprKind::Binary(_, left, right) => {
                ids.extend(collect_node_ids(left));
                ids.extend(collect_node_ids(right));
            }
            ExprKind::If(cond, then_block, else_expr) => {
                ids.extend(collect_node_ids(cond));
                ids.push(then_block.id);
                if let Some(else_expr) = else_expr {
                    ids.extend(collect_node_ids(else_expr));
                }
            }
            ExprKind::Block(block) => {
                ids.push(block.id);
                for stmt in block.stmts.iter() {
                    ids.push(stmt.id);
                    // Could recursively collect from stmt contents too
                }
            }
            ExprKind::Paren(inner) => {
                ids.extend(collect_node_ids(inner));
            }
            ExprKind::Return(maybe_expr) => {
                if let Some(expr) = maybe_expr {
                    ids.extend(collect_node_ids(expr));
                }
            }
            ExprKind::Lit(_) | ExprKind::Path(_) | ExprKind::Err => {
                // These don't contain other expressions
            }
        }

        ids
    }

    fn parse(src: &str, filename: &str, skip_output: bool) -> anyhow::Result<()> {
        let (token_iter, lex_errs) = scrap_lexer::Token::lexer(src).spanned().fold(
            (Vec::new(), Vec::new()),
            |(mut tokens, mut token_errors), (new_tok, new_span)| {
                let span = SimpleSpan::from(new_span);
                match new_tok {
                    // Turn the `Range<usize>` spans logos gives us into chumsky's `SimpleSpan` via `Into`, because it's easier
                    // to work with
                    Ok(new_tok) => tokens.push((new_tok, span)),
                    Err(e) => token_errors.push(Rich::<Token, _>::custom(span, e.to_string())),
                }

                (tokens, token_errors)
            },
        );

        // Turn the token iterator into a stream that chumsky can use for things like backtracking
        let token_stream = Stream::from_iter(token_iter)
            // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
            // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
            .map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

        let (_ast, parse_errs) = file_parser().parse(token_stream).into_output_errors();

        if parse_errs.is_empty() && lex_errs.is_empty() {
            return Ok(());
        }

        if skip_output {
            return Err(anyhow::anyhow!("parse error"));
        }

        // Use the enhanced error reporting system
        crate::error::report_parse_errors(parse_errs, lex_errs, filename, src);

        return Err(anyhow::anyhow!("parse error"));
    }

    #[test]
    fn parse_basic_function() -> anyhow::Result<()> {
        let filename = "basic.sc";
        let src = std::fs::read_to_string(format!("../../example/{}", filename))?;
        parse(&src, filename, false)
    }

    #[test]
    fn test_ast() -> anyhow::Result<()> {
        let filename = "test.sc";
        let src = TEST_AST;
        parse(&src, filename, false)
    }

    #[test]
    fn test_return_statements() -> anyhow::Result<()> {
        let filename = "test_return.sc";
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
        parse(&src, filename, false)
    }

    #[test]
    fn test_simple_return_functionality() -> anyhow::Result<()> {
        let filename = "simple_return_test.sc";
        let src = std::fs::read_to_string(format!("../../example/{}", filename))?;
        parse(&src, filename, false)
    }

    #[test]
    fn test_return_requires_semicolon() {
        let filename = "test_return_no_semicolon.sc";
        let src = r#"
        fn test_return_no_semicolon() -> i32 {
            return 42
        }
        "#;
        
        // This should fail because return statement lacks semicolon
        let result = parse(&src, filename, true);
        assert!(result.is_err(), "Expected parse error for return statement without semicolon");
    }

    #[test]
    fn test_return_with_semicolon_works() -> anyhow::Result<()> {
        let filename = "test_return_with_semicolon.sc";
        let src = r#"
        fn test_return_with_semicolon() -> i32 {
            return 42;
        }
        
        fn test_void_return() {
            return;
        }
        "#;
        
        // This should pass because return statements have semicolons
        parse(&src, filename, false)
    }

    #[test]
    fn test_missing_semicolon_error_quality() {
        let filename = "test_missing_semicolon.sc";
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
        
        println!("Testing this source code:");
        println!("{}", src);
        
        // Let's see what happens when we parse this
        let result = parse(&src, filename, false);
        
        if result.is_err() {
            println!("✓ Parse correctly failed with error");
        } else {
            println!("✗ Parse unexpectedly succeeded - this should be an error!");
            panic!("Expected parse error for missing semicolon");
        }
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
        
        let result1 = parse(&test1, "test1.sc", false);
        assert!(result1.is_err(), "Expected error for missing semicolon after simple let");

        // Test case 2: Missing semicolon after function call
        let test2 = r#"
        fn test() {
            foo()  // Missing semicolon
            bar();
        }
        "#;
        
        let result2 = parse(&test2, "test2.sc", false);
        assert!(result2.is_err(), "Expected error for missing semicolon after function call");

        // Test case 3: Missing semicolon after return statement  
        let test3 = r#"
        fn test() -> i32 {
            return 42  // Missing semicolon
        }
        "#;
        
        let result3 = parse(&test3, "test3.sc", false);
        assert!(result3.is_err(), "Expected error for missing semicolon after return");
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
            let result = parse(test_case, &format!("correct_{}.sc", i), true);
            assert!(result.is_ok(), "Expected successful parse for correct case {}", i);
        }
    }
}
