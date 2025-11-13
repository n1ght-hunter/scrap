use scrap_span::Span;
use strum_macros::{EnumDiscriminants, EnumIter};
use thin_vec::ThinVec;

use crate::{
    Visibility, enumdef::EnumDef, fndef::FnDef, ident::Ident, module::Module, node_id::NodeId,
    path::Path, structdef::StructDef,
};

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Item<'db> {
    pub kind: ItemKind<'db>,
    pub span: Span<'db>,
    pub id: NodeId,
    pub vis: Visibility<'db>,
}

#[derive(
    Debug,
    Clone,
    EnumDiscriminants,
    Hash,
    PartialEq,
    Eq,
    salsa::Update,
    serde::Serialize,
    serde::Deserialize,
)]
#[strum_discriminants(derive(EnumIter))]
pub enum ItemKind<'db> {
    Fn(FnDef<'db>),
    Enum(EnumDef<'db>),
    Struct(StructDef<'db>),
    Module(Ident<'db>, Module<'db>),
    Use(UseTree<'db>),
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct UseTree<'db> {
    pub prefix: Path<'db>,
    pub kind: UseTreeKind<'db>,
    pub span: Span<'db>,
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum UseTreeKind<'db> {
    Simple(Option<Ident<'db>>),
    Nested {
        items: ThinVec<UseTree<'db>>,
        span: Span<'db>,
    },
    Glob,
}
