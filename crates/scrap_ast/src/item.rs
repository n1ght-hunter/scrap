use scrap_span::Span;
use strum_macros::{EnumDiscriminants, EnumIter};
use thin_vec::ThinVec;

use crate::{
    Visibility, enumdef::EnumDef, fndef::FnDef, module::Module, node_id::NodeId,
    structdef::StructDef,
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
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        self.kind.pretty_print(f)
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
    Module(Path<'db>, Module<'db>),
    Use(UseTree<'db>),
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for ItemKind<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        match self {
            ItemKind::Fn(fndef) => fndef.pretty_print(f),
            ItemKind::Enum(enumdef) => enumdef.pretty_print(f),
            ItemKind::Struct(structdef) => structdef.pretty_print(f),
            ItemKind::Module(path, module) => {
                write!(f, "mod {} ", {
                    let mut s = String::new();
                    path.pretty_print(&mut s).unwrap();
                    s
                })?;
                module.pretty_print(f)
            }
            ItemKind::Use(use_tree) => {
                write!(f, "use ")?;
                use_tree.pretty_print(f)
            }
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
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        self.prefix.pretty_print(f)?;
        match &self.kind {
            UseTreeKind::Simple(alias) => {
                if let Some(alias_ident) = alias {
                    write!(f, " as ")?;
                    alias_ident.pretty_print(f)?;
                }
                Ok(())
            }
            UseTreeKind::Nested { items, span: _ } => {
                write!(f, "::{{")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    item.pretty_print(f)?;
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
