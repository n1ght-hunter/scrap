use scrap_span::Span;
use strum_macros::{EnumDiscriminants, EnumIter};
use thin_vec::ThinVec;

use crate::{
    Visibility, enumdef::EnumDef, fndef::FnDef, foreign::ForeignMod, module::Module,
    node_id::NodeId, structdef::StructDef,
};
use scrap_shared::{ident::Ident, path::Path};

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Item<'db> {
    pub kind: ItemKind<'db>,
    pub span: Span<'db>,
    pub id: NodeId,
    pub vis: Visibility<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Item<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        Self::write_indent(f, indent)?;
        self.kind.pretty_print_indent(f, indent)
    }
}

#[derive(
    Debug,
    Clone,
    EnumDiscriminants,
    Hash,
    PartialEq,
    Eq,
    salsa::Update,
    serde::Serialize,
    serde::Deserialize,
)]
#[strum_discriminants(derive(EnumIter))]
pub enum ItemKind<'db> {
    Fn(FnDef<'db>),
    Enum(EnumDef<'db>),
    Struct(StructDef<'db>),
    Module(Module<'db>),
    Use(UseTree<'db>),
    ForeignMod(ForeignMod<'db>),
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for ItemKind<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        match self {
            ItemKind::Fn(fndef) => fndef.pretty_print_indent(f, indent),
            ItemKind::Enum(enumdef) => enumdef.pretty_print_indent(f, indent),
            ItemKind::Struct(structdef) => structdef.pretty_print_indent(f, indent),
            ItemKind::Module(module) => {
                salsa::with_attached_database(|db| {
                    write!(f, "mod {} ", module.id(db).path(db))
                })
                .unwrap_or_else(|| write!(f, "mod <no db> "))?;
                module.pretty_print_indent(f, indent)
            }
            ItemKind::Use(use_tree) => {
                write!(f, "use ")?;
                use_tree.pretty_print_indent(f, indent)?;
                write!(f, ";")
            }
            ItemKind::ForeignMod(foreign_mod) => foreign_mod.pretty_print_indent(f, indent),
        }
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct UseTree<'db> {
    pub prefix: Path<'db>,
    pub kind: UseTreeKind<'db>,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for UseTree<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        self.prefix.pretty_print_indent(f, 0)?;
        match &self.kind {
            UseTreeKind::Simple(alias) => {
                if let Some(alias_ident) = alias {
                    write!(f, " as ")?;
                    alias_ident.pretty_print_indent(f, 0)?;
                }
                Ok(())
            }
            UseTreeKind::Nested { items, span: _ } => {
                write!(f, "::{{")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    item.pretty_print_indent(f, 0)?;
                }
                write!(f, "}}")
            }
            UseTreeKind::Glob => {
                write!(f, "::*")
            }
        }
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum UseTreeKind<'db> {
    Simple(Option<Ident<'db>>),
    Nested {
        items: ThinVec<UseTree<'db>>,
        span: Span<'db>,
    },
    Glob,
}
