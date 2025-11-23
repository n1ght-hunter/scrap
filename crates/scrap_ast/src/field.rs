use scrap_span::Span;

use crate::{Visibility, node_id::NodeId, typedef::Ty};
use scrap_shared::ident::Ident;

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct FieldDef<'db> {
    pub id: NodeId,
    pub span: Span<'db>,
    pub vis: Visibility<'db>,
    pub ident: Option<Ident<'db>>,
    pub ty: Box<Ty<'db>>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for FieldDef<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        if let Some(ident) = &self.ident {
            ident.pretty_print(f)?;
            write!(f, ": ")?;
        }
        self.ty.pretty_print(f)
    }
}
