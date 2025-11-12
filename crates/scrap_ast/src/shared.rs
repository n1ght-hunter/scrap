use scrap_errors::ErrorGuaranteed;
use scrap_span::Span;

use crate::{node_id::NodeId, path::Path};

#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub struct Visibility<'db> {
    pub kind: VisibilityKind<'db>,
    pub span: Span<'db>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub enum VisibilityKind<'db> {
    Public,
    Restricted {
        path: Box<Path<'db>>,
        id: NodeId,
        shorthand: bool,
    },
    Inherited,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize)]
pub enum Recovered {
    No,
    Yes(ErrorGuaranteed),
}
