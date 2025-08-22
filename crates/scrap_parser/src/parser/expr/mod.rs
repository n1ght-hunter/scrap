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
pub use atom::{literal_parser, identifier_parser, parenthesized_parser, atom_with_recovery};
pub use block_expr::block_expr_parser;
pub use call::call_parser;
pub use if_expr::if_expr_parser;
pub use inline::{expr_parser, inline_expr_parser};
pub use items::items_parser;

// Re-export binary operations from parent module
pub use super::binary::{bin_op_parser, product_parser, sum_parser, comparison_parser};

/// An expression. Following Rust AST structure.
#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
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
/// This is a subset of the full Rust ExprKind enum.
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// Array(ThinVec<Box<Expr>>) - simplified as Array
    Array(LocalVec<Box<Expr>>),
    
    /// Call(Box<Expr>, ThinVec<Box<Expr>>) - function call
    Call(Box<Expr>, LocalVec<Box<Expr>>),
    
    /// Binary(BinOp, Box<Expr>, Box<Expr>) - binary operation
    Binary(BinOp, Box<Expr>, Box<Expr>),
    
    /// Lit(Lit) - literal
    Lit(Lit),
    
    /// If(Box<Expr>, Box<Block>, Option<Box<Expr>>) - if expression
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    
    /// Block(Box<Block>, Option<Label>) - simplified to Block(Box<Block>)
    Block(Box<Block>),
    
    /// Path(Option<Box<QSelf>>, Path) - simplified to Path(String)
    Path(String),
    
    /// Paren(Box<Expr>) - parenthesized expression
    Paren(Box<Expr>),
    
    /// Placeholder for expressions that weren't syntactically well formed
    Err,
}

// Module re-exports
