use crate::{node_id::NodeId, path::Path};
use scrap_errors::ErrorGuaranteed;
use scrap_span::Span;
use thin_vec::ThinVec;

#[derive(Debug, Clone)]
pub struct Ty {
    pub id: NodeId,
    pub kind: TyKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TyKind {
    Path(Path),
    Tup(ThinVec<Box<Ty>>),
    Dummy,
    Never,
    Err(ErrorGuaranteed),
}
