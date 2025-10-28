use std::collections::HashSet;

use crate::{ast::NodeId, parse_error::ParseError, utils::LocalVec};

use super::{
    ScrapInput, ScrapParser,
    ident::Ident,
    parse_ident,
    typedef::{Type, parse_type},
};
use chumsky::prelude::*;
use scrap_lexer::Token;

#[derive(Debug, Clone)]
pub struct Field {
    pub id: NodeId,
    pub ident: Ident,
    pub ty: Type,
}

/// will emit an warn or error fi the field name starts with uppcase
pub fn fields<'tokens, I>() -> impl ScrapParser<'tokens, I, LocalVec<Field>>
where
    I: ScrapInput<'tokens>,
{
    parse_ident()
        .then_ignore(just(Token::Colon))
        .then(parse_type())
        .map_with(|(ident, ty), e| Field {
            id: e.state().new_node_id(),
            ident,
            ty,
        })
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<LocalVec<_>>()
        .validate(move |args, _, emitter| {
            let mut field_name = HashSet::new();

            args.iter().for_each(|field| {
                if !field_name.insert(&field.ident.name) {
                    emitter.emit(ParseError::custom(
                        field.ident.span,
                        format!("duplicate identifier '{}'", field.ident.name),
                    ));
                }
            });
            args
        })
}
