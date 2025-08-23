use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{ast::NodeId, utils::LocalVec};

use super::{
    ScrapInput, ScrapParser, capital_ident,
    field::{Field, fields},
    ident::Ident,
    parse_ident,
};

#[derive(Debug, Clone)]
pub struct StructDef {
    pub id: NodeId,
    pub ident: Ident,
    pub fields: LocalVec<Field>,
}

pub fn struct_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, StructDef>
where
    I: ScrapInput<'tokens, 'src>,
{
    let fields = fields(true)
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .labelled("struct fields");

    just(Token::Struct)
        .ignore_then(
            capital_ident("Struct name must start with an uppercase letter")
                .labelled("struct name"),
        )
        .then(fields)
        .map_with(|(name, fields), e| StructDef {
            id: e.state().new_node_id(),
            ident: name,
            fields,
        })
        .labelled("struct")
}
