use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{node_id::NodeId, stmt::Stmt};

/// A block expression. Following Rust AST structure.
#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub struct Block<'db> {
    pub stmts: ThinVec<Stmt<'db>>,
    pub id: NodeId,
    pub span: Span<'db>,
}
