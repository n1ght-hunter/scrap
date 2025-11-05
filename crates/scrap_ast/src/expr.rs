use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{
    block::Block,
    lit::Lit,
    node_id::NodeId,
    operators::{AssignOp, BinOp},
    path::Path,
};

/// An expression node in the AST
#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

/// Expression kinds - subset of Rust's ExprKind enum
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// An array literal (e.g., `[a, b, c, d]`)
    Array(ThinVec<Box<Expr>>),
    /// A function call
    Call(Box<Expr>, ThinVec<Box<Expr>>),
    /// A binary operation (e.g., `a + b`, `a * b`)
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// A literal value (e.g., `1`, `"foo"`)
    Lit(Lit),
    /// An `if` block, with an optional `else` block
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    /// A block (`{ ... }`)
    Block(Box<Block>),
    /// Variable reference
    Path(Path),
    /// A parenthesized expression
    Paren(Box<Expr>),
    /// A `return` expression
    Return(Option<Box<Expr>>),
    /// An assignment (`place = expr`)
    Assign(Box<Expr>, Box<Expr>, Span),
    /// An assignment with an operator (`place += expr`)
    AssignOp(AssignOp, Box<Expr>, Box<Expr>),
    /// Error placeholder
    Err,
}
