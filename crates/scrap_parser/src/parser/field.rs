use std::collections::HashSet;

use crate::{ast::NodeId, utils::LocalVec, Span};

use super::{ident::Ident, parse_ident, typedef::{parse_type, Type}};
use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;



#[derive(Debug, Clone)]
pub struct Field {
    pub id: NodeId,
    pub ident: Ident,
    pub ty: Type,
}


pub fn fields<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, LocalVec<Field>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parse_ident()
        .then_ignore(just(Token::Colon))
        .then(parse_type())
        .map(|(ident, ty)| Field { 
            id: NodeId::new(),
            ident, 
            ty 
        })
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<LocalVec<_>>()
        .validate(|args, _, emitter| {
            let mut field_name = HashSet::new();

            args.iter().for_each(|field| {
                if !field_name.insert(&field.ident.name) {
                    emitter.emit(Rich::custom(
                        field.ident.span,
                        format!("duplicate identifier '{}'", field.ident.name),
                    ));
                }
            });
            args
        })
}
