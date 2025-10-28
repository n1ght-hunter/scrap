use chumsky::prelude::*;
use inflections::Inflect;
use scrap_lexer::Token;

use crate::{ast::NodeId, parse_error::ParseError};

use super::{
    ScrapInput, ScrapParser, field::Field, ident::Ident, parse_ident, typedef::parse_type,
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

/// Parse an enum definition
pub fn enum_parser<'tokens, I>() -> impl ScrapParser<'tokens, I, EnumDef>
where
    I: ScrapInput<'tokens>,
{
    let variant = parse_ident()
        .validate(move |id, _, emitter| {
            if !id.name.is_pascal_case() {
                emitter.emit(ParseError::custom(
                    id.span,
                    format!(
                        "Enum variant name must be in PascalCase: {} -> {}",
                        id.name,
                        id.name.to_pascal_case()
                    ),
                ));
            }

            id
        })
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
            parse_ident()
                .validate(move |id, _, emitter| {
                    if !id.name.is_pascal_case() {
                        emitter.emit(ParseError::custom(
                            id.span,
                            format!(
                                "Enum name must be in PascalCase: {} -> {}",
                                id.name,
                                id.name.to_pascal_case()
                            ),
                        ));
                    }

                    id
                })
                .labelled("enum name"),
        )
        .then(variant.delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map_with(|(name, variants), e| EnumDef {
            id: e.state().new_node_id(),
            ident: name,
            variants,
        })
        .labelled("enum")
}
