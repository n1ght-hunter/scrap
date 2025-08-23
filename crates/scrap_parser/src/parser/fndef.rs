use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{ast::NodeId, utils::LocalVec, Span};

use super::{
    block::{Block, block_parser},
    field::{Field, fields},
    ident::Ident,
    parse_ident,
    typedef::{Type, parse_type},
    ScrapParser, ScrapInput, // Import our new traits
};

#[derive(Debug, Clone)]
pub struct FnDef {
    pub id: NodeId,
    pub ident: Ident,
    pub args: LocalVec<Field>,
    pub ret_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

pub fn function_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, FnDef>
where
    I: ScrapInput<'tokens, 'src>,
{
    let args = fields()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .labelled("function args");

    just(Token::Fn)
        .ignore_then(parse_ident().labelled("function name"))
        .then(args)
        .map_with(|start, e| (start, e.span()))
        .then(just(Token::Arrow).ignore_then(parse_type()).or_not().labelled("return type"))
        .then(block_parser())
        .map_with(|((((name, args), span), ret_type), body), _| FnDef {
            id: NodeId::new(),
            ident: name,
            args,
            ret_type,
            body,
            span,
        })
        .labelled("function")
}
