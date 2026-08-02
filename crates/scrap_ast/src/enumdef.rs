use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{field::FieldDef, generics::Generics, node_id::NodeId};
use scrap_shared::ident::Ident;

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct EnumDef {
    pub id: NodeId,
    pub ident: Ident,
    pub generics: Generics,
    pub variants: Vec<Variant>,
}

impl scrap_shared::pretty_print::PrettyPrint for EnumDef {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        write!(f, "enum {}", {
            let mut s = String::new();
            self.ident.pretty_print(&mut s).unwrap();
            s
        })?;
        self.generics.pretty_print(f)?;
        writeln!(f, " {{")?;
        for variant in &self.variants {
            write!(f, "    ")?;
            variant.pretty_print(f)?;
            writeln!(f, ",")?;
        }
        write!(f, "}}")
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct Variant {
    pub id: NodeId,
    pub span: Span,
    // pub vis: Visibility,
    pub ident: Ident,
    pub data: VariantData,
}

impl scrap_shared::pretty_print::PrettyPrint for Variant {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
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
    Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub enum VariantData {
    Struct { fields: ThinVec<FieldDef> },
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
