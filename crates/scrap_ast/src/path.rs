use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{ident::Ident, node_id::NodeId};

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub struct PathSegment<'db> {
    /// The identifier portion of this path segment.
    pub ident: Ident<'db>,
    /// The unique ID of this path segment.
    pub id: NodeId,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub struct Path<'db> {
    pub span: Span<'db>,
    /// The segments in the path: the things separated by `::`.
    /// Global paths begin with `kw::PathRoot`.
    pub segments: ThinVec<PathSegment<'db>>,
}

impl<'db> Path<'db> {
    pub fn from_ident(ident: Ident<'db>) -> Self {
        let id = ident.id;
        Path {
            span: ident.span,
            segments: ThinVec::from([PathSegment { ident, id }]),
        }
    }
}
