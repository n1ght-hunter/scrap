use crate::{node_id::NodeId, path::Path};
use scrap_errors::ErrorGuaranteed;
use scrap_span::Span;
use thin_vec::ThinVec;

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update)]
pub struct Ty<'db> {
    pub id: NodeId,
    pub kind: TyKind<'db>,
    pub span: Span<'db>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update)]
pub enum TyKind<'db> {
    Path(Path<'db>),
    Tup(ThinVec<Box<Ty<'db>>>),
    Dummy,
    Never,
    Err(ErrorGuaranteed),
}
