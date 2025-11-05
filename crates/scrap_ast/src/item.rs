use scrap_span::Span;

use crate::{enumdef::EnumDef, fndef::FnDef, node_id::NodeId, structdef::StructDef};

#[derive(Debug, Clone)]
pub struct Item<K = ItemKind> {
    pub kind: K,
    pub span: Span,
    pub id: NodeId,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Fn(FnDef),
    Enum(EnumDef),
    Struct(StructDef),
}
