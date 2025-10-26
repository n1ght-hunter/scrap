use crate::{Span, ast::NodeId};
use chumsky::prelude::*;
use scrap_lexer::Token;

use super::{
    block::Block, expr::{expr_parser, Expr}, pat::{pat_parser, Pat}, typedef::{parse_type, Type}, ScrapInput, ScrapParser
};

/// Local represents a `let` statement, e.g., `let <pat>:<ty> = <expr>;`.
#[derive(Debug, Clone)]
pub struct Local {
    pub id: NodeId,
    pub super_: Option<Span>,
    pub pat: Box<Pat>,
    pub ty: Option<Type>,
    pub expr: Box<Expr>,
    pub span: Span,
}

pub fn parse_local<'tokens, 'src: 'tokens, I>(
    block_parser: impl ScrapParser<'tokens, 'src, I, Block> + 'tokens,
) -> impl ScrapParser<'tokens, 'src, I, Local>
where
    I: ScrapInput<'tokens, 'src>,
{
    just(Token::Let)
        .ignore_then(pat_parser())
        .then(just(Token::Colon).ignore_then(parse_type()).or_not())
        .then_ignore(just(Token::Assign))
        .then(expr_parser(block_parser))
        .then_ignore(just(Token::Semicolon))
        .map_with(|((pat, ty), expr), e| Local {
            id: e.state().new_node_id(),
            super_: None,
            pat: Box::new(pat),
            ty,
            expr: Box::new(expr),
            span: e.span(),
        })
}
