use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::Span;

use super::{capital_ident, field::Field, ident::Ident, typedef::parse_type};

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Unit(Ident),
    Full(Field),
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub ident: Ident,
    pub variants: Vec<EnumVariant>,
}

pub fn enum_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, EnumDef, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let err_msg = "Enum variant name must start with an uppercase letter";

    let variant = capital_ident(err_msg)
        .then(
            parse_type()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .or_not(),
        )
        .map(|(ident, ty)| {
            if let Some(ty) = ty {
                EnumVariant::Full(Field { ident, ty })
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
        .map(|(name, variants)| EnumDef {
            ident: name,
            variants,
        })
        .labelled("enum")
}
