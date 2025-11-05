use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{ident::Ident, node_id::NodeId};

#[derive(Debug, Clone)]
pub struct PathSegment {
    /// The identifier portion of this path segment.
    pub ident: Ident,
    /// The unique ID of this path segment.
    pub id: NodeId,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub span: Span,
    /// The segments in the path: the things separated by `::`.
    /// Global paths begin with `kw::PathRoot`.
    pub segments: ThinVec<PathSegment>,
}

impl Path {
    pub fn from_ident(ident: Ident) -> Self {
        let id = ident.id;
        Path {
            span: ident.span,
            segments: ThinVec::from([PathSegment { ident, id }]),
        }
    }
}