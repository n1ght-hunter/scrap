use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{ast::NodeId, utils::LocalVec};

use super::{
    ScrapInput, // Import our new traits
    ScrapParser,
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
