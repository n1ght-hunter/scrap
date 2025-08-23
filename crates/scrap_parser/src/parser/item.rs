use crate::{Span, ast::NodeId};
use chumsky::prelude::*;

use super::{
    ScrapInput, // Import our new traits
    ScrapParser,
    enumdef::{EnumDef, enum_parser},
    fndef::{FnDef, function_parser},
    structdef::{StructDef, struct_parser},
};

#[derive(Debug, Clone)]
pub struct Item<K = ItemKind> {
    pub kind: K,
    pub span: Span,
    pub id: NodeId,
    // pub vis: Visibility,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Fn(FnDef),
    Enum(EnumDef),
    Struct(StructDef),
}

/// Parse a sc file into ast
pub fn item_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Item>
where
    I: ScrapInput<'tokens, 'src>,
{
    choice((
        function_parser().map(ItemKind::Fn),
        enum_parser().map(ItemKind::Enum),
        struct_parser().map(ItemKind::Struct),
    ))
    .map_with(|kind, e| Item {
        kind,
        span: e.span(),
        id: NodeId::from_u32(0), // TODO: use state
    })
}
