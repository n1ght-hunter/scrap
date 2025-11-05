use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{block::Block, ident::Ident, node_id::NodeId, pat::Pat, typedef::Ty};

#[derive(Debug, Clone)]
pub struct FnDef {
    pub id: NodeId,
    pub ident: Ident,
    pub args: ThinVec<Param>,
    pub ret_type: Option<Ty>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub id: NodeId,
    pub ident: Ident,
    pub ty: Box<Ty>,
    pub pat: Box<Pat>,
    pub span: Span,
}
