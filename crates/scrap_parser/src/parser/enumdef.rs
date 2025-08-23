use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::ast::NodeId;

use super::{
    ScrapInput, ScrapParser, capital_ident, field::Field, ident::Ident, typedef::parse_type,
};

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Unit(Ident),
    Full(Field),
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub id: NodeId,
    pub ident: Ident,
    pub variants: Vec<EnumVariant>,
}

pub fn enum_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, EnumDef>
where
    I: ScrapInput<'tokens, 'src>,
{
    let err_msg = "Enum variant name must start with an uppercase letter";

    let variant = capital_ident(err_msg)
        .then(
            parse_type()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .or_not(),
        )
        .map_with(|(ident, ty), e| {
            if let Some(ty) = ty {
                EnumVariant::Full(Field {
                    id: e.state().new_node_id(),
                    ident,
                    ty,
                })
            } else {
                EnumVariant::Unit(ident)
            }
        })
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>();

    just(Token::Enum)
        .ignore_then(
            capital_ident("Enum name must start with an uppercase letter").labelled("enum name"),
        )
        .then(variant.delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map_with(|(name, variants), e| EnumDef {
            id: e.state().new_node_id(),
            ident: name,
            variants,
        })
        .labelled("enum")
}
