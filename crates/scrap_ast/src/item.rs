use scrap_span::Span;
use strum_macros::{EnumDiscriminants, EnumIter};

use crate::{enumdef::EnumDef, fndef::FnDef, ident::Ident, module::Module, node_id::NodeId, structdef::StructDef};

#[derive(Debug, Clone)]
pub struct Item<K = ItemKind> {
    pub kind: K,
    pub span: Span,
    pub id: NodeId,
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
pub enum ItemKind {
    Fn(FnDef),
    Enum(EnumDef),
    Struct(StructDef),
    Module(Ident, Module),
}

