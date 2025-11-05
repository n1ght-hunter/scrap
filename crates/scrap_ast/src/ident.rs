use scrap_span::{Span, Symbol};

use crate::node_id::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ident {
    pub id: NodeId,
    pub name: Symbol,
    pub span: Span,
}
