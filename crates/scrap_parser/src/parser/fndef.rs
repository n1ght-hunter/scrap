use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId};

use super::{
    block::{Block, block_parser},
    field::{Field, fields},
    ident::Ident,
    parse_ident,
    typedef::{Type, parse_type},
};

#[derive(Debug, Clone)]
pub struct FnDef {
    pub id: NodeId,
    pub ident: Ident,
    pub args: Vec<Field>,
    pub ret_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

pub fn function_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, FnDef, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let args = fields()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .labelled("function args");

    just(Token::Fn)
        .ignore_then(parse_ident().labelled("function name"))
        .then(args)
        .map_with(|start, e| (start, e.span()))
        .then_ignore(just(Token::Arrow))
        .then(parse_type().or_not().labelled("return type"))
        .then(block_parser())
        .map_with(|((((name, args), span), ret_type), body), _| FnDef {
            id: NodeId::new(),
            ident: name,
            args: args.into_iter().collect(),
            ret_type,
            body,
            span,
        })
        .labelled("function")
}
