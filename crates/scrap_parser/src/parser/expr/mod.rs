//! Expression parsing module
//!
//! This module contains all expression-related parsers organized into separate files:
//! 
//! ## Module Organization
//! - **`atom.rs`**: Atomic expressions (literals, identifiers, paths, parenthesized expressions)
//! - **`block_expr.rs`**: Block expressions enclosed in braces
//! - **`call.rs`**: Function call expressions with argument parsing
//! - **`if_expr.rs`**: If-else expressions with optional branches
//! - **`inline.rs`**: Main expression parsers with proper precedence handling and coordination
//! - **`items.rs`**: Parser for comma-separated expression lists
//!
//! ## Key Features
//! 
//! ### Proper Operator Precedence
//! The parsers implement correct operator precedence following mathematical conventions:
//! 1. **Function calls** (highest precedence)
//! 2. **Multiplication and Division** (`*`, `/`)
//! 3. **Addition and Subtraction** (`+`, `-`)
//! 4. **Comparison operations** (`>`, `<`, `>=`, `<=`, `==`, `!=`) (lowest precedence)
//!
//! ### Error Recovery
//! - Graceful handling of malformed expressions
//! - Nested delimiter recovery for balanced parentheses/braces/brackets
//! - Skip-and-retry recovery for syntax errors
//!
//! ### Recursive Structure
//! - Proper handling of nested expressions
//! - Parenthesized expressions for precedence override
//! - Support for complex expression trees
//!
//! ## Usage
//! 
//! The main entry points are:
//! - `expr_parser()`: Full expression parser with all features
//! - `inline_expr_parser()`: Simplified parser for basic expressions

use super::{binary::BinOp, block::Block, lit::Lit};
use crate::{Span, ast::NodeId, utils::LocalVec};

pub mod atom;
pub mod block_expr;
pub mod call;
pub mod if_expr;
pub mod inline;
pub mod items;

// Re-export the main parser functions
pub use atom::{literal_parser, identifier_parser, parenthesized_parser, return_parser, atom_with_recovery};
pub use block_expr::block_expr_parser;
pub use call::call_parser;
pub use if_expr::if_expr_parser;
pub use inline::{expr_parser, inline_expr_parser};
pub use items::items_parser;

// Re-export binary operations from parent module
pub use super::binary::{bin_op_parser, product_parser, sum_parser, comparison_parser};

/// An expression. Following Rust AST structure exactly.
/// 
/// An expression is a piece of code that evaluates to a value.
/// In Rust, almost everything is an expression, including blocks,
/// if statements, function calls, and more.
#[derive(Debug, Clone)]
pub struct Expr {
    /// Unique identifier for this expression node
    pub id: NodeId,
    /// The specific kind of expression
    pub kind: ExprKind,
    /// Source location span for this expression
    pub span: Span,
}

impl Expr {
    /// Create a new expression with the given kind and span
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self {
            id: NodeId::new(),
            kind,
            span,
        }
    }
}

/// Expression kinds, following Rust AST enum structure exactly.
/// This is a subset of the full Rust ExprKind enum, containing
/// only the most essential expression types for our language.
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// An array literal (e.g., `[a, b, c, d]`).
    /// Contains a list of expressions that make up the array elements.
    Array(LocalVec<Box<Expr>>),
    
    /// A function call.
    /// The first field resolves to the function itself,
    /// and the second field is the list of arguments.
    /// This also represents calling the constructor of
    /// tuple-like ADTs such as tuple structs and enum variants.
    Call(Box<Expr>, LocalVec<Box<Expr>>),
    
    /// A binary operation (e.g., `a + b`, `a * b`).
    /// Contains the operator and the left and right operands.
    Binary(BinOp, Box<Expr>, Box<Expr>),
    
    /// A literal value (e.g., `1`, `"foo"`).
    /// This includes numbers, strings, booleans, etc.
    Lit(Lit),
    
    /// An `if` block, with an optional `else` block.
    /// `if expr { block } else { expr }`
    /// If present, the "else" expr is always `ExprKind::Block` (for `else`) or
    /// `ExprKind::If` (for `else if`).
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    
    /// A block (`{ ... }`).
    /// Blocks are expressions that contain a sequence of statements
    /// and optionally evaluate to the value of their final expression.
    Block(Box<Block>),
    
    /// Variable reference, possibly containing `::` and/or type
    /// parameters (e.g., `foo::bar::<baz>`).
    /// Simplified from the full Rust Path type for our needs.
    Path(String),
    
    /// A parenthesized expression.
    /// No-op: used solely so we can pretty-print faithfully.
    /// Preserves the original parentheses in the source code.
    Paren(Box<Expr>),
    
    /// A `return` expression.
    /// `return` or `return expr` where expr is the optional value to return.
    /// If no expression is provided, it returns the unit type `()`.
    Return(Option<Box<Expr>>),
    
    /// Placeholder for expressions that weren't syntactically well formed.
    /// This is used for error recovery during parsing.
    Err,
}

// Module re-exports
