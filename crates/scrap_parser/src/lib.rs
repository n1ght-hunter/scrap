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

use parser::{Parser, State};
use scrap_ast::module::{Module, ModuleKind};
use scrap_errors::ErrorGuaranteed;
use scrap_lexer::{Token, token_stream::TokenStreamCursor};
use scrap_shared::path::Path;

mod errors;
pub mod parser;
mod utils;

pub type PResult<'a, T> = std::result::Result<T, ErrorGuaranteed>;
pub type TokenStream<'db> = scrap_lexer::token_stream::TokenStream<'db>;

#[salsa::tracked(debug, persist)]
pub struct ParsedFile<'db> {
    #[returns(ref)]
    /// The AST representation of this file (either a Can or a Module)
    pub ast: CanOrModule<'db>,
    #[returns(ref)]
    /// All modules defined in this file
    pub modules: Vec<scrap_ast::module::Module<'db>>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum CanOrModule<'db> {
    Can(scrap_ast::Can<'db>),
    Module(scrap_ast::module::Module<'db>),
}

impl<'db> CanOrModule<'db> {
    pub fn unwrap_can(&self) -> &scrap_ast::Can<'db> {
        match self {
            CanOrModule::Can(can) => can,
            CanOrModule::Module(_) => panic!("Expected Can, found Module"),
        }
    }

    pub fn unwrap_module(&self) -> &scrap_ast::module::Module<'db> {
        match self {
            CanOrModule::Module(module) => module,
            CanOrModule::Can(_) => panic!("Expected Module, found Can"),
        }
    }

    pub fn to_module(&self, db: &'db dyn scrap_shared::Db) -> scrap_ast::module::Module<'db> {
        match self {
            CanOrModule::Module(module) => module.clone(),
            CanOrModule::Can(can) => scrap_ast::module::Module::new(
                db,
                can.name(db).clone(),
                ModuleKind::Loaded(
                    can.items(db).clone(),
                    scrap_ast::module::Inline::Yes,
                    scrap_span::new_dummy_span(db),
                ),
            ),
        }
    }
}

#[salsa::tracked(persist)]
pub fn parse_tokens<'db>(
    db: &'db dyn scrap_shared::Db,
    file: scrap_shared::salsa::InputFile<'db>,
    tokens: scrap_lexer::LexedTokens<'db>,
    is_root: bool,
    root_path: Vec<String>,
) -> Option<ParsedFile<'db>> {
    let tokens = tokens.tokens(db);
    let token_stream = TokenStreamCursor::new(tokens);
    let state = State::new(file.path(db).to_str().unwrap());
    let path = Path::from_segments(db, &root_path);
    let id = scrap_shared::id::ModuleId::new(db, path.clone());
    let mut parser = Parser::new(db, file.content(db), token_stream, state, path.clone());
    let ast = if is_root {
        parser.parse_can().map(|ast| {
            ParsedFile::new(
                db,
                CanOrModule::Can(ast),
                std::mem::take(&mut parser.modules),
            )
        })
    } else {
        parser.parse_module_inner(Token::Eof).map(|ast| {
            ParsedFile::new(
                db,
                CanOrModule::Module(Module::new(
                    db,
                    id,
                    ModuleKind::Loaded(
                        ast,
                        scrap_ast::module::Inline::No,
                        scrap_span::new_dummy_span(db),
                    ),
                )),
                std::mem::take(&mut parser.modules),
            )
        })
    };
    ast.ok()
}

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
