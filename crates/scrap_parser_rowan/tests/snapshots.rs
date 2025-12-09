//! Snapshot tests for the rowan-based parser.
//!
//! These tests ensure the CST output remains consistent across changes.
//! The same test inputs are used in scrap_parser for AST comparison.

use scrap_parser_rowan::{SyntaxNode, parse_file};
use scrap_lexer::lex_file;

fn parse(db: &dyn scrap_shared::Db, source: &str) -> SyntaxNode {
    let file = scrap_shared::salsa::InputFile::new(db, "test.sc".into(), source.into());
    let tokens = lex_file(db, file).expect("lexing failed");
    let parsed = parse_file(db, file, tokens);
    parsed.syntax(db)
}

#[scrap_macros::salsa_test]
fn simple_function(db: &dyn scrap_shared::Db) {
    let source = r#"fn main() {
    let x = 42;
}"#;
    insta::assert_debug_snapshot!(parse(db, source));
}

#[scrap_macros::salsa_test]
fn function_with_params(db: &dyn scrap_shared::Db) {
    let source = r#"fn add(a: int, b: int) -> int {
    a + b
}"#;
    insta::assert_debug_snapshot!(parse(db, source));
}

#[scrap_macros::salsa_test]
fn struct_def(db: &dyn scrap_shared::Db) {
    let source = r#"struct Point {
    x: int,
    y: int,
}"#;
    insta::assert_debug_snapshot!(parse(db, source));
}

#[scrap_macros::salsa_test]
fn enum_def(db: &dyn scrap_shared::Db) {
    let source = r#"enum Color {
    Red,
    Green,
    Blue(int),
}"#;
    insta::assert_debug_snapshot!(parse(db, source));
}

#[scrap_macros::salsa_test]
fn if_expr(db: &dyn scrap_shared::Db) {
    let source = r#"fn test() -> int {
    if x > 10 {
        42
    } else {
        0
    }
}"#;
    insta::assert_debug_snapshot!(parse(db, source));
}

#[scrap_macros::salsa_test]
fn module_def(db: &dyn scrap_shared::Db) {
    let source = r#"mod inner {
    fn helper() {
        print("hello");
    }
}"#;
    insta::assert_debug_snapshot!(parse(db, source));
}

#[scrap_macros::salsa_test]
fn use_statement(db: &dyn scrap_shared::Db) {
    let source = r#"use foo::bar;
use baz::qux;"#;
    insta::assert_debug_snapshot!(parse(db, source));
}

#[scrap_macros::salsa_test]
fn complex(db: &dyn scrap_shared::Db) {
    let source = r#"struct Config {
    name: String,
    value: int,
}

enum Result {
    Ok(int),
    Err,
}

fn process(cfg: Config) -> Result {
    if cfg.value > 0 {
        return Result::Ok(cfg.value);
    }
    Result::Err
}"#;
    insta::assert_debug_snapshot!(parse(db, source));
}
