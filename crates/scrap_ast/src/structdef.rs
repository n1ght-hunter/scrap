use crate::{enumdef::VariantData, ident::Ident, node_id::NodeId};

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update)]
pub struct StructDef<'db> {
    pub id: NodeId,
    pub ident: Ident<'db>,
    pub data: VariantData<'db>,
}
