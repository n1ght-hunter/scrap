use scrap_span::Span;

use crate::{Visibility, ident::Ident, node_id::NodeId, typedef::Ty};

#[derive(Debug, Clone, Hash, PartialEq, Eq, salsa::Update)]
pub struct FieldDef<'db> {
    pub id: NodeId,
    pub span: Span<'db>,
    pub vis: Visibility<'db>,
    pub ident: Option<Ident<'db>>,
    pub ty: Box<Ty<'db>>,
}
