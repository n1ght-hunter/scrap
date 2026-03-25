use scrap_span::Span;

use crate::{fndef::FnDef, node_id::NodeId};
use scrap_shared::ident::Ident;

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct ImplBlock<'db> {
    pub id: NodeId,
    pub type_name: Ident<'db>,
    pub methods: Vec<FnDef<'db>>,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for ImplBlock<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        write!(f, "impl ")?;
        self.type_name.pretty_print_indent(f, 0)?;
        writeln!(f, " {{")?;
        salsa::with_attached_database(|_| {
            for method in &self.methods {
                Self::write_indent(f, indent + 1)?;
                method.pretty_print_indent(f, indent + 1)?;
                writeln!(f)?;
            }
            Ok(())
        })
        .unwrap_or(Ok(()))?;
        Self::write_indent(f, indent)?;
        write!(f, "}}")
    }
}
