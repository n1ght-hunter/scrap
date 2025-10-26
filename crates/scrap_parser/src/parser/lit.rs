//! Literal value parsing and representation
//!
//! This module handles the parsing and representation of literal values in the language.
//! The types follow the Rust AST structure for representing literals.

use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{ast::NodeId, parse_error::ParseError};

use super::{ScrapInput, ScrapParser};

/// A literal value with its kind and actual data.
/// This represents any literal value that appears in source code.
#[derive(Debug, Clone)]
pub struct Lit {
    /// Unique identifier for this literal node
    pub id: NodeId,
    /// The kind of literal (determines how it should be interpreted)
    pub kind: LitKind,
    /// The actual literal data (simplified representation for our language)
    pub temp_lit: TempLit,
    // In full Rust AST, there would also be:
    // pub symbol: Symbol,        // The original source representation
    // pub suffix: Option<Symbol>, // Type suffix like "f32" in "1.0f32"
}

/// Temporary literal representation for our simplified language.
/// This holds the actual parsed values rather than symbols/tokens.
#[derive(Debug, Clone)]
pub enum TempLit {
    /// A boolean literal (`true`, `false`)
    Bool(bool),
    /// An integer literal (`1`, `42`, `-5`)
    Int(i64),
    /// A floating-point literal (`1.0`, `3.14`)
    Float(f64),
    /// A string literal (`"hello"`, `"world"`)
    Str(String),
}

/// Literal kinds, following Rust AST enum structure.
/// This is a simplified subset of the full Rust LitKind enum.
///
/// Note that the entire literal (including the suffix) is considered when
/// deciding the `LitKind`. This means that float literals like `1f32` are
/// classified by this type as `Float`. This is different to `token::LitKind`
/// which does *not* consider the suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LitKind {
    /// A boolean literal (`true`, `false`)
    Bool,
    /// An integer literal (`1`)
    Integer,
    /// A float literal (`1.0`, `1f64` or `1E10f64`)
    Float,
    /// A string literal (`"foo"`). The symbol is unescaped, and so may differ
    /// from the original token's symbol.
    Str,
}

/// Parse literal values from tokens into AST nodes.
///
/// This parser handles all supported literal types in our language:
/// - Boolean literals (`true`, `false`)
/// - Integer literals (`1`, `42`, `-5`)
/// - Floating-point literals (`1.0`, `3.14`)
/// - String literals (`"hello"`, `"world"`)
///
/// The parser extracts the value from the token and creates the appropriate
/// `Lit` node with the correct `LitKind` and `TempLit` representation.
pub fn lit_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Lit>
where
    I: ScrapInput<'tokens, 'src>,
{
    select! {
        Token::Bool(value) => (LitKind::Bool, TempLit::Bool(value)),
        Token::Int(value) => (LitKind::Integer, TempLit::Int(value)),
        Token::Float(value) => (LitKind::Float, TempLit::Float(value)),
        Token::Str(value) => (LitKind::Str, TempLit::Str(value.to_string())),
    }
    .map_with(
        |(kind, temp_lit),
         e: &mut chumsky::input::MapExtra<
            '_,
            '_,
            I,
            extra::Full<ParseError<'tokens, Token<'src>>, crate::parser::State, ()>,
        >| Lit {
            id: e.state().new_node_id(),
            kind,
            temp_lit,
        },
    )
    .labelled("literal")
}
