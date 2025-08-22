use crate::{Span, ast::NodeId};
use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use super::{
    expr::{Expr, inline_expr_parser},
    pat::{Pat, pat_parser},
    typedef::{Type, parse_type},
};

/// Local represents a `let` statement, e.g., `let <pat>:<ty> = <expr>;`.
#[derive(Debug, Clone)]
pub struct Local {
    pub id: NodeId,
    pub super_: Option<Span>,
    pub pat: Box<Pat>,
    pub ty: Option<Type>,
    pub kind: LocalKind,
    pub span: Span,
    pub colon_sp: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum LocalKind {
    /// Local declaration.
    /// Example: `let x;`
    Decl,
    /// Local declaration with an initializer.
    /// Example: `let x = y;`
    Init(Expr),
}

pub fn parse_local<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Local, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    just(Token::Let)
        .ignore_then(pat_parser())
        .then(just(Token::Colon).ignore_then(parse_type()).or_not())
        .then_ignore(just(Token::Assign))
        .then(inline_expr_parser())
        .map_with(|((pat, ty), expr), e| Local {
            id: NodeId::new(),
            super_: None,
            pat: Box::new(pat),
            ty: ty,
            kind: LocalKind::Init(expr),
            span: e.span(),
            colon_sp: None,
        })
}
