use scrap_errors::ErrorGuaranteed;
use scrap_shared::path::Path;
use scrap_span::Span;

pub use scrap_shared::NodeId;

#[derive(
    Clone, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Visibility<'db> {
    pub kind: VisibilityKind<'db>,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Visibility<'db> {
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
pub enum VisibilityKind<'db> {
    Public,
    Restricted {
        path: Box<Path<'db>>,
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
