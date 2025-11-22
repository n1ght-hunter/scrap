use scrap_shared::id::ModuleId;
use scrap_span::Span;
use thin_vec::ThinVec;

use crate::item::Item;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Inline {
    Yes,
    No,
}

#[salsa::tracked(debug, persist)]
pub struct Module<'db> {
    pub id: ModuleId<'db>,
    #[returns(ref)]
    pub kind: ModuleKind<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Module<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        salsa::with_attached_database(|db| self.kind(db).pretty_print(f))
            .unwrap_or_else(|| f.write_str("<no db>"))?;

        Ok(())
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum ModuleKind<'db> {
    Loaded(ThinVec<Box<Item<'db>>>, Inline, Span<'db>),
    Unloaded,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for ModuleKind<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        match self {
            ModuleKind::Loaded(items, _, _) => {
                write!(f, "{{")?;
                for item in items.iter() {
                    item.pretty_print(f)?;
                }
                write!(f, "}}")
            }
            ModuleKind::Unloaded => write!(f, "<unloaded module>"),
        }
    }
}
