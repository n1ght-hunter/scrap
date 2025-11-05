use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{node_id::NodeId, stmt::Stmt};

/// A block expression. Following Rust AST structure.
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: ThinVec<Stmt>,
    pub id: NodeId,
    pub span: Span,
    pub error: bool,
}
