use chumsky::prelude::*;
use inflections::Inflect;
use scrap_lexer::Token;

use crate::{ast::NodeId, parse_error::ParseError, utils::LocalVec};

use super::{
    ScrapInput, ScrapParser,
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

/// Parse a struct definition
pub fn struct_parser<'tokens, I>() -> impl ScrapParser<'tokens, I, StructDef>
where
    I: ScrapInput<'tokens>,
{
    let fields = fields()
        .validate(|fields, _, emmiter| {
            for field in fields.iter() {
                if !field.ident.name.is_snake_case() {
                    // Maybe should be a hard error
                    emmiter.emit(crate::parse_error::ParseError::custom_with_kind(
                        field.ident.span,
                        format!(
                            "fields should be in snake_case: {} -> {}",
                            field.ident.name,
                            field.ident.name.to_snake_case()
                        ),
                        crate::parse_error::kind::ReportKind::Warning,
                    ));
                }
            }
            fields
        })
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .labelled("struct fields");

    just(Token::Struct)
        .ignore_then(
            parse_ident()
                .validate(move |id, _, emitter| {
                    if !id.name.is_pascal_case() {
                        emitter.emit(ParseError::custom(
                            id.span,
                            format!(
                                "Struct name must be in PascalCase: {} -> {}",
                                id.name,
                                id.name.to_pascal_case()
                            ),
                        ));
                    }

                    id
                })
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
