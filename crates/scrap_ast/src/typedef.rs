use crate::node_id::NodeId;
use scrap_errors::ErrorGuaranteed;
use scrap_shared::{path::Path, pretty_print::PrettyPrint};
use scrap_span::Span;
use thin_vec::ThinVec;

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Ty<'db> {
    pub id: NodeId,
    pub kind: TyKind<'db>,
    pub span: Span<'db>,
}

impl<'db> PrettyPrint for Ty<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        self.kind.pretty_print(f)
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub enum TyKind<'db> {
    Path(Path<'db>),
    Tup(ThinVec<Box<Ty<'db>>>),
    Dummy,
    Never,
    Err(ErrorGuaranteed),
}

impl<'db> PrettyPrint for TyKind<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        match self {
            TyKind::Path(path) => path.pretty_print(f),
            TyKind::Tup(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    ty.pretty_print(f)?;
                }
                write!(f, ")")
            }
            TyKind::Dummy => write!(f, "<dummy type>"),
            TyKind::Never => write!(f, "!"),
            TyKind::Err(_) => write!(f, "<error type>"),
        }
    }
}
