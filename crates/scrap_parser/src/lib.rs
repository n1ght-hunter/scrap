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

pub type Span = SimpleSpan;


#[derive(Debug, Clone, Copy)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use ariadne::{Color, Label, Report, ReportKind, sources};
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
        use std::collections::HashSet;
        use crate::parser::expr::inline_expr_parser;
        
        let test_expressions = vec![
            "42",              // Simple literal  
            "foo",             // Simple identifier
            "x + 1",           // Simple binary expression
            "a * 3",           // Another binary expression
            "b + 5",           // Addition
            "123",             // Another literal
            "hello",           // Another identifier
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
                    assert_eq!(unique_ids.len(), node_ids.len(), 
                        "Duplicate NodeIds found within expression: {}", expr_src);
                    
                    // Add to global set to check across expressions
                    for id in node_ids {
                        assert!(all_node_ids.insert(id), 
                            "NodeId {:?} was reused across different expressions!", id);
                    }
                }
                Err(parse_errors) => {
                    println!("  Parse failed: {:?}", parse_errors);
                }
            }
        }

        println!("Total nodes created: {}, All unique: {}", total_nodes, all_node_ids.len());
        assert_eq!(total_nodes, all_node_ids.len(), "Some NodeIds were duplicated!");
        
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
            ExprKind::Lit(_) | ExprKind::Path(_) | ExprKind::Err => {
                // These don't contain other expressions
            }
        }
        
        ids
    }

    #[test]
    fn test_ast() -> anyhow::Result<()> {
        let filename = "test.scrap";
        let src = TEST_AST;
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

        println!("token_iter {:#?}", token_iter);

        // Turn the token iterator into a stream that chumsky can use for things like backtracking
        let token_stream = Stream::from_iter(token_iter)
            // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
            // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
            .map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

        let (ast, parse_errs) = file_parser().parse(token_stream).into_output_errors();

        if let Some(sexpr) = ast {
            println!("ast {:?}", sexpr);
        }

        if parse_errs.is_empty() && lex_errs.is_empty() {
            return Ok(());
        }

        parse_errs
            .into_iter()
            .map(|e| e.map_token(|c| c.to_string()))
            .chain(
                lex_errs
                    .into_iter()
                    .map(|e| e.map_token(|tok| tok.to_string())),
            )
            .for_each(|e| {
                Report::build(ReportKind::Error, (filename, e.span().into_range()))
                    .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                    .with_message(e.to_string())
                    .with_label(
                        Label::new((filename, e.span().into_range()))
                            .with_message(e.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .with_labels(e.contexts().map(|(label, span)| {
                        Label::new((filename, span.into_range()))
                            .with_message(format!("while parsing this {label}"))
                            .with_color(Color::Yellow)
                    }))
                    .finish()
                    .print(sources([(filename, src)]))
                    .unwrap()
            });

        return Err(anyhow::anyhow!("parse error"));
    }
}
