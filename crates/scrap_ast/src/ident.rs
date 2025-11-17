use scrap_span::{Span, Symbol};

use crate::node_id::NodeId;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Ident<'db> {
    pub id: NodeId,
    pub name: Symbol<'db>,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Ident<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let name = salsa::with_attached_database(|db| self.name.text(db).to_owned())
            .unwrap_or("<invalid>".to_string());
        write!(f, "{}", name)
    }
}

impl<'db> Ident<'db> {
    pub fn dummy(db: &'db dyn scrap_shared::Db) -> Self {
        Self {
            id: NodeId::dummy(),
            name: Symbol::dummy(db),
            span: Span::new_default(db),
        }
    }

    pub fn dummy_with_name(db: &'db dyn scrap_shared::Db, name: &str) -> Self {
        Self {
            id: NodeId::dummy(),
            name: Symbol::new(db, name),
            span: Span::new_default(db),
        }
    }
}
