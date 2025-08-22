use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId};

use super::{
    field::{Field, fields},
    ident::Ident,
    parse_ident,
};

#[derive(Debug, Clone)]
pub struct StructDef {
    pub id: NodeId,
    pub ident: Ident,
    pub fields: Vec<Field>,
}

pub fn struct_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, StructDef, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let fields = fields()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .labelled("struct fields");

    just(Token::Struct)
        .ignore_then(parse_ident().labelled("struct name"))
        .then(fields)
        .map(|(name, fields)| StructDef {
            id: NodeId::new(),
            ident: name,
            fields,
        })
        .labelled("struct")
}
