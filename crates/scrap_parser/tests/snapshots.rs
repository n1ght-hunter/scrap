//! Snapshot tests for the AST-producing parser.
//!
//! These tests ensure the AST output remains consistent across changes.
//! The same test inputs are used in scrap_parser_rowan for CST comparison.

use scrap_parser::{CanOrModule, parse_tokens};
use scrap_lexer::lex_file;
use scrap_test_utils::{salsa_assert_snapshot, salsa_test};

fn parse<'db>(db: &'db dyn scrap_shared::Db, source: &str) -> CanOrModule<'db> {
    let file = scrap_shared::salsa::InputFile::new(db, "test.sc".into(), source.into());
    let tokens = lex_file(db, file).expect("lexing failed");
    let parsed = parse_tokens(db, file, tokens, true, vec!["test".to_string()])
        .expect("parsing failed");
    parsed.ast(db).clone()
}

#[salsa_test]
fn simple_function(db: &dyn scrap_shared::Db) {
    let source = r#"fn main() {
    let x = 42;
}"#;
    salsa_assert_snapshot!("simple_function", parse(db, source));
}

#[salsa_test]
fn function_with_params(db: &dyn scrap_shared::Db) {
    let source = r#"fn add(a: int, b: int) -> int {
    a + b
}"#;
    salsa_assert_snapshot!("function_with_params", parse(db, source));
}

#[salsa_test]
fn struct_def(db: &dyn scrap_shared::Db) {
    let source = r#"struct Point {
    x: int,
    y: int,
}"#;
    salsa_assert_snapshot!("struct_def", parse(db, source));
}

#[salsa_test]
fn enum_def(db: &dyn scrap_shared::Db) {
    let source = r#"enum Color {
    Red,
    Green,
    Blue(int),
}"#;
    salsa_assert_snapshot!("enum_def", parse(db, source));
}

#[salsa_test]
fn if_expr(db: &dyn scrap_shared::Db) {
    let source = r#"fn test() -> int {
    if x > 10 {
        42
    } else {
        0
    }
}"#;
    salsa_assert_snapshot!("if_expr", parse(db, source));
}

#[salsa_test]
fn module_def(db: &dyn scrap_shared::Db) {
    let source = r#"mod inner {
    fn helper() {
        print("hello");
    }
}"#;
    salsa_assert_snapshot!("module_def", parse(db, source));
}

#[salsa_test]
fn use_statement(db: &dyn scrap_shared::Db) {
    let source = r#"use foo::bar;
use baz::qux;"#;
    salsa_assert_snapshot!("use_statement", parse(db, source));
}

#[salsa_test]
#[ignore = "lexer has issues with this test case"]
fn complex(db: &dyn scrap_shared::Db) {
    let source = "struct Config {\n    name: str,\n    value: int,\n}\n\nenum MyResult {\n    Ok(int),\n    Err,\n}\n\nfn process(cfg: Config) -> int {\n    if cfg.value > 0 {\n        return cfg.value;\n    }\n    0\n}";
    salsa_assert_snapshot!("complex", parse(db, source));
}
