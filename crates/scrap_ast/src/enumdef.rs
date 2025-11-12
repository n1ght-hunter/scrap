use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{field::FieldDef, ident::Ident, node_id::NodeId};

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub struct EnumDef<'db> {
    pub id: NodeId,
    pub ident: Ident<'db>,
    pub variants: Vec<Variant<'db>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub struct Variant<'db> {
    pub id: NodeId,
    pub span: Span<'db>,
    // pub vis: Visibility,
    pub ident: Ident<'db>,
    pub data: VariantData<'db>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub enum VariantData<'db> {
    Struct { fields: ThinVec<FieldDef<'db>> },
    Tuple(ThinVec<FieldDef<'db>>, NodeId),
    Unit(NodeId),
}

impl<'db> VariantData<'db> {
    pub fn is_struct(&self) -> bool {
        matches!(self, VariantData::Struct { .. })
    }

    pub fn unwrap_struct(&self) -> &ThinVec<FieldDef<'db>> {
        if let VariantData::Struct { fields } = self {
            fields
        } else {
            panic!("called `unwrap_struct()` on a non-struct VariantData");
        }
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, VariantData::Tuple(_, _))
    }

    pub fn unwrap_tuple(&self) -> &ThinVec<FieldDef<'db>> {
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
