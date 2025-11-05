use scrap_span::Span;

use crate::{Visibility, ident::Ident, node_id::NodeId, typedef::Ty};



#[derive(Debug, Clone)]
pub struct FieldDef {
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Option<Ident>,
    pub ty: Box<Ty>,
}