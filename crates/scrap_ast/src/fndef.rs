use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{block::Block, ident::Ident, node_id::NodeId, pat::Pat, typedef::Ty};

#[salsa::tracked(debug)]
pub struct FnDef<'db> {
    pub id: NodeId,
    pub ident: Ident<'db>,
    #[tracked]
    #[returns(ref)]
    pub args: ThinVec<Param<'db>>,
    #[tracked]
    #[returns(ref)]
    pub ret_type: Option<Ty<'db>>,
    #[tracked]
    #[returns(ref)]
    pub body: Block<'db>,
    pub span: Span<'db>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update)]
pub struct Param<'db> {
    pub id: NodeId,
    pub ident: Ident<'db>,
    pub ty: Box<Ty<'db>>,
    pub pat: Box<Pat<'db>>,
    pub span: Span<'db>,
}
