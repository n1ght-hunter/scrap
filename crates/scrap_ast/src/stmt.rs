use scrap_span::Span;

use crate::{expr::Expr, item::Item, local::Local, node_id::NodeId};

/// A statement. Following Rust AST structure exactly.
///
/// No `attrs` or `tokens` fields because each `StmtKind` variant
/// contains an AST node with those fields. (Except for `StmtKind::Empty`,
/// which never has attrs or tokens)
#[derive(Debug, Clone)]
pub struct Stmt {
    /// Unique identifier for this statement node
    pub id: NodeId,
    /// The specific kind of statement
    pub kind: StmtKind,
    /// Source location span for this statement
    pub span: Span,
}

/// Statement kinds, following Rust AST enum structure exactly.
/// This is a subset of the full Rust StmtKind enum.
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// A local (let) binding (e.g., `let <pat>:<ty> = <expr>;`).
    Let(Box<Local>),

    /// An item definition (e.g., function, struct, etc.).
    Item(Box<Item>),
    /// Expr without trailing semi-colon.
    Expr(Box<Expr>),
    /// Expr with a trailing semi-colon.
    Semi(Box<Expr>),
    /// Just a trailing semi-colon.
    Empty,
}
