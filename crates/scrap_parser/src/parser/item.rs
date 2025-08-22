use crate::{Span, ast::NodeId};
use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use super::{
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
pub fn item_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Item, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    function_parser()
        .map(ItemKind::Fn)
        .or(enum_parser().map(ItemKind::Enum))
        .or(struct_parser().map(ItemKind::Struct))
        .map_with(|kind, e| Item {
            kind,
            span: e.span(),
            id: NodeId::new(),
            // vis: Visibility::Public, // Default visibility, can be changed later
        })
}
