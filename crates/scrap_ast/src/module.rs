use scrap_shared::id::ModuleId;
use scrap_span::Span;
use thin_vec::ThinVec;

use crate::item::Item;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
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
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        salsa::with_attached_database(|db| self.kind(db).pretty_print_indent(f, indent))
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
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        match self {
            ModuleKind::Loaded(items, _, _) => {
                writeln!(f, "{{")?;
                for item in items.iter() {
                    item.pretty_print_indent(f, indent + 1)?;
                    writeln!(f)?;
                }
                Self::write_indent(f, indent)?;
                write!(f, "}}")
            }
            ModuleKind::Unloaded => write!(f, "<unloaded module>"),
        }
    }
}
