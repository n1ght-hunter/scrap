use scrap_span::{Span, Symbol};

use crate::node_id::NodeId;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Ident<'db> {
    pub id: NodeId,
    pub name: Symbol<'db>,
    pub span: Span<'db>,
}
