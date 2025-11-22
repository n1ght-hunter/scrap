use crate::{enumdef::VariantData, node_id::NodeId};
use scrap_shared::ident::Ident;

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct StructDef<'db> {
    pub id: NodeId,
    pub ident: Ident<'db>,
    pub data: VariantData<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for StructDef<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(f, "struct {} ", self.ident.pretty_to_string())?;
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
