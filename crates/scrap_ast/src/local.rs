use scrap_shared::pretty_print::PrettyPrint;
use scrap_span::Span;

use crate::{expr::Expr, node_id::NodeId, pat::Pat, typedef::Ty};

/// Local represents a `let` statement, e.g., `let <pat>:<ty> = <expr>;`.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Local<'db> {
    pub id: NodeId,
    pub pat: Box<Pat<'db>>,
    pub ty: Option<Ty<'db>>,
    pub kind: LocalKind<'db>,
    pub span: Span<'db>,
}

impl<'db> PrettyPrint for Local<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        write!(f, "let ")?;
        self.pat.pretty_print_indent(f, 0)?;
        if let Some(ty) = &self.ty {
            write!(f, ": ")?;
            ty.pretty_print_indent(f, 0)?;
        }
        match &self.kind {
            LocalKind::Decl => {}
            LocalKind::Init(expr) => {
                write!(f, " = ")?;
                expr.pretty_print_indent(f, 0)?;
            }
        }
        write!(f, ";")
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
/// The kind of local variable declaration.
pub enum LocalKind<'db> {
    /// a declaration like `let x;`
    Decl,
    /// an initialization like `let x = expr;`
    Init(Box<Expr<'db>>),
}
