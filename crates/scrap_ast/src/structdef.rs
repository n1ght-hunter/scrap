use crate::{enumdef::VariantData, ident::Ident, node_id::NodeId};

#[derive(Debug, Clone)]
pub struct StructDef {
    pub id: NodeId,
    pub ident: Ident,
    pub data: VariantData,
}
