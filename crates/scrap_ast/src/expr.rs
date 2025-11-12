use scrap_span::Span;
use strum_macros::{EnumDiscriminants, EnumIter};
use thin_vec::ThinVec;

use crate::{
    block::Block,
    lit::Lit,
    node_id::NodeId,
    operators::{AssignOp, BinOp},
    path::Path,
};

/// An expression node in the AST
#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub struct Expr<'db> {
    pub id: NodeId,
    pub kind: ExprKind<'db>,
    pub span: Span<'db>,
}

/// Expression kinds - subset of Rust's ExprKind enum
#[derive(Debug, Clone, Hash, PartialEq, Eq, EnumDiscriminants, salsa::Update, serde::Serialize, serde::Deserialize)]
#[strum_discriminants(derive(EnumIter))]
pub enum ExprKind<'db> {
    /// An array literal (e.g., `[a, b, c, d]`)
    Array(ThinVec<Box<Expr<'db>>>),
    /// A function call
    Call(Box<Expr<'db>>, ThinVec<Box<Expr<'db>>>),
    /// A binary operation (e.g., `a + b`, `a * b`)
    Binary(BinOp<'db>, Box<Expr<'db>>, Box<Expr<'db>>),
    /// A literal value (e.g., `1`, `"foo"`)
    Lit(Lit<'db>),
    /// An `if` block, with an optional `else` block
    If(Box<Expr<'db>>, Box<Block<'db>>, Option<Box<Expr<'db>>>),
    /// A block (`{ ... }`)
    Block(Box<Block<'db>>),
    /// Variable reference
    Path(Path<'db>),
    /// A parenthesized expression
    Paren(Box<Expr<'db>>),
    /// A `return` expression
    Return(Option<Box<Expr<'db>>>),
    /// An assignment (`place = expr`)
    Assign(Box<Expr<'db>>, Box<Expr<'db>>, Span<'db>),
    /// An assignment with an operator (`place += expr`)
    AssignOp(AssignOp<'db>, Box<Expr<'db>>, Box<Expr<'db>>),
    /// Error placeholder
    Err,
}
