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

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Error,
    Path(String),
    Call(Box<Expr>, LocalVec<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Lit(Lit),
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
}

// Module re-exports
