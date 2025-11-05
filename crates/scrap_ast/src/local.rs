use scrap_span::Span;

use crate::{expr::Expr, node_id::NodeId, pat::Pat, typedef::Ty};

/// Local represents a `let` statement, e.g., `let <pat>:<ty> = <expr>;`.
#[derive(Debug, Clone)]
pub struct Local {
    pub id: NodeId,
    pub pat: Box<Pat>,
    pub ty: Option<Ty>,
    pub expr: Box<Expr>,
    pub span: Span,
}
