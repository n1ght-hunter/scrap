use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{field::FieldDef, ident::Ident, node_id::NodeId};

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct EnumDef<'db> {
    pub id: NodeId,
    pub ident: Ident<'db>,
    pub variants: Vec<Variant<'db>>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for EnumDef<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(f, "enum {} {{\n", {
            let mut s = String::new();
            self.ident.pretty_print(&mut s).unwrap();
            s
        })?;
        for variant in &self.variants {
            write!(f, "    ")?;
            variant.pretty_print(f)?;
            write!(f, ",\n")?;
        }
        write!(f, "}}")
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Variant<'db> {
    pub id: NodeId,
    pub span: Span<'db>,
    // pub vis: Visibility,
    pub ident: Ident<'db>,
    pub data: VariantData<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Variant<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        self.ident.pretty_print(f)?;
        match &self.data {
            VariantData::Struct { fields } => {
                write!(f, " {{ ")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    field.pretty_print(f)?;
                }
                write!(f, " }}")
            }
            VariantData::Tuple(fields, _) => {
                write!(f, "(")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    field.pretty_print(f)?;
                }
                write!(f, ")")
            }
            VariantData::Unit(_) => Ok(()),
        }
    }
}

#[derive(
    Clone, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
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
