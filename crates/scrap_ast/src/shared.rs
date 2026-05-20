use scrap_errors::ErrorGuaranteed;
use scrap_shared::path::Path;
use scrap_span::Span;

pub use scrap_shared::NodeId;

#[derive(
    Clone, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Visibility {
    pub kind: VisibilityKind,
    pub span: Span,
}

impl scrap_shared::pretty_print::PrettyPrint for Visibility {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        match &self.kind {
            VisibilityKind::Public => write!(f, "pub"),
            VisibilityKind::Restricted { path, .. } => {
                write!(f, "pub(")?;
                path.pretty_print_indent(f, 0)?;
                write!(f, ")")
            }
            VisibilityKind::Inherited => Ok(()),
        }
    }
}

#[derive(
    Clone, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum VisibilityKind {
    Public,
    Restricted {
        path: Box<Path>,
        id: NodeId,
        shorthand: bool,
    },
    Inherited,
}

#[derive(
    Clone, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum Recovered {
    No,
    Yes(ErrorGuaranteed),
}
