use scrap_span::Span;

use crate::{Db, NodeId, pretty_print::PrettyPrint};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Ident<'db> {
    pub id: NodeId,
    pub name: Symbol<'db>,
    pub span: Span<'db>,
}

impl<'db> PrettyPrint for Ident<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        let name = salsa::with_attached_database(|db| self.name.text(db).to_owned())
            .unwrap_or("<invalid>".to_string());
        write!(f, "{}", name)
    }
}

impl<'db> Ident<'db> {
    pub fn dummy(db: &'db dyn Db) -> Self {
        Self {
            id: NodeId::dummy(),
            name: Symbol::dummy(db),
            span: Span::new_default(db),
        }
    }

    pub fn dummy_with_name(db: &'db dyn Db, name: &str) -> Self {
        Self {
            id: NodeId::dummy(),
            name: Symbol::new(db, name),
            span: Span::new_default(db),
        }
    }
}

/// A symbol represents an interned string.
#[salsa::interned(debug, persist)]
pub struct Symbol<'db> {
    #[returns(ref)]
    pub text: String,
}

impl<'db> Symbol<'db> {
    /// Get the string slice for this symbol.
    pub fn dummy(db: &'db dyn Db) -> Self {
        Symbol::new(db, "<dummy>")
    }

    pub fn as_bits(&self) -> u64 {
        self.0.as_bits()
    }
}
