use scrap_span::Span;
use strum_macros::{EnumDiscriminants, EnumIter};

use crate::{enumdef::EnumDef, fndef::FnDef, ident::Ident, module::Module, node_id::NodeId, structdef::StructDef};

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update)]
pub struct Item<'db> {
    pub kind: ItemKind<'db>,
    pub span: Span<'db>,
    pub id: NodeId,
}

#[derive(Debug, Clone, EnumDiscriminants, Hash, PartialEq, Eq, salsa::Update)]
#[strum_discriminants(derive(EnumIter))]
pub enum ItemKind<'db> {
    Fn(FnDef<'db>),
    Enum(EnumDef<'db>),
    Struct(StructDef<'db>),
    Module(Ident<'db>, Module<'db>),
}

