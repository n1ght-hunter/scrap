use scrap_shared::{ident::Ident, path::Path, pretty_print::PrettyPrint};
use scrap_span::Span;
use thin_vec::ThinVec;

use crate::node_id::NodeId;

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, Default, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Generics {
    pub params: ThinVec<GenericParam>,
}

impl Generics {
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct GenericParam {
    pub id: NodeId,
    pub ident: Ident,
    pub kind: GenericParamKind,
    pub bounds: ThinVec<GenericBound>,
    pub span: Span,
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum GenericParamKind {
    Type,
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum GenericBound {
    Trait(Path),
}

impl PrettyPrint for Generics {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        if self.params.is_empty() {
            return Ok(());
        }
        write!(f, "<")?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            param.ident.pretty_print(f)?;
        }
        write!(f, ">")
    }
}
