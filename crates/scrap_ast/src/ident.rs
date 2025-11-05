use scrap_span::Span;

use crate::node_id::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub id: NodeId,
    pub name: String,
    pub span: Span,
}
