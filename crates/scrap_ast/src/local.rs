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
