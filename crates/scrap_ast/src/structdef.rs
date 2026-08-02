use crate::{enumdef::VariantData, generics::Generics, node_id::NodeId};
use scrap_shared::ident::Ident;

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct StructDef {
    pub id: NodeId,
    pub ident: Ident,
    pub generics: Generics,
    pub data: VariantData,
}

impl scrap_shared::pretty_print::PrettyPrint for StructDef {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        write!(f, "struct {}", self.ident.pretty_to_string())?;
        self.generics.pretty_print(f)?;
        write!(f, " ")?;
        match &self.data {
            VariantData::Struct { fields } => {
                write!(f, "{{ ")?;
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
