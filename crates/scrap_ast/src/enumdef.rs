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
    },
    Tuple(ThinVec<FieldDef>, NodeId),
    Unit(NodeId),
}

impl VariantData {
    pub fn is_struct(&self) -> bool {
        matches!(self, VariantData::Struct { .. })
    }

    pub fn unwrap_struct(&self) -> &ThinVec<FieldDef> {
        if let VariantData::Struct { fields } = self {
            fields
        } else {
            panic!("called `unwrap_struct()` on a non-struct VariantData");
        }
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, VariantData::Tuple(_, _))
    }

    pub fn unwrap_tuple(&self) -> &ThinVec<FieldDef> {
        if let VariantData::Tuple(fields, _) = self {
            fields
        } else {
            panic!("called `unwrap_tuple()` on a non-tuple VariantData");
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, VariantData::Unit(_))
    }

    pub fn unwrap_unit(&self) -> NodeId {
        if let VariantData::Unit(id) = self {
            *id
        } else {
            panic!("called `unwrap_unit()` on a non-unit VariantData");
        }
    }
}