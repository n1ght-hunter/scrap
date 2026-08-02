use crate::node_id::NodeId;
use scrap_errors::ErrorGuaranteed;
use scrap_shared::{path::Path, pretty_print::PrettyPrint, types::Mutability};
use scrap_span::Span;
use thin_vec::ThinVec;

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct Ty {
    pub id: NodeId,
    pub kind: TyKind,
    pub span: Span,
}

impl PrettyPrint for Ty {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        self.kind.pretty_print(f)
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub enum TyKind {
    Path(Path),
    Tup(ThinVec<Box<Ty>>),
    /// A reference type: `&T` or `&mut T`
    Ref(Box<Ty>, Mutability),
    /// A GC-managed pointer type: `*T`
    Ptr(Box<Ty>),
    Dummy,
    Never,
    Err(ErrorGuaranteed),
}

impl PrettyPrint for TyKind {
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
            TyKind::Ref(inner, mutability) => {
                write!(f, "{}", mutability.ref_prefix_str())?;
                inner.pretty_print(f)
            }
            TyKind::Ptr(inner) => {
                write!(f, "*")?;
                inner.pretty_print(f)
            }
            TyKind::Dummy => write!(f, "<dummy type>"),
            TyKind::Never => write!(f, "!"),
            TyKind::Err(_) => write!(f, "<error type>"),
        }
    }
}
