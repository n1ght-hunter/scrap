use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{Recovered, field::FieldDef, ident::Ident, node_id::NodeId};

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub id: NodeId,
    pub ident: Ident,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub id: NodeId,
    pub span: Span,
    // pub vis: Visibility,
    pub ident: Ident,
    pub data: VariantData,
}

#[derive(Clone, Debug)]
pub enum VariantData {
    Struct {
        fields: ThinVec<FieldDef>,
        recovered: Recovered,
    },
    Tuple(ThinVec<FieldDef>, NodeId),
    Unit(NodeId),
}
