use scrap_errors::ErrorGuaranteed;
use scrap_span::Span;

use crate::{node_id::NodeId, path::Path};

#[derive(Clone, Debug)]
pub struct Visibility {
    pub kind: VisibilityKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum VisibilityKind {
    Public,
    Restricted {
        path: Box<Path>,
        id: NodeId,
        shorthand: bool,
    },
    Inherited,
}


#[derive(Clone, Debug)]
pub enum Recovered {
    No,
    Yes(ErrorGuaranteed),
}