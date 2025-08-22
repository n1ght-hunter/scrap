use super::{ident::Ident, parse_ident};
use crate::{Span, ast::NodeId};
use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

#[derive(Debug, Clone)]
pub struct Ty {
    pub id: NodeId,
    pub kind: TyKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TyKind {}

#[derive(Debug, Clone)]
pub struct Type(pub Ident);

pub fn parse_type<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Type, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parse_ident().map_with(|ident, _| Type(ident))
}
