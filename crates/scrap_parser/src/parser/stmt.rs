use crate::{Span, ast::NodeId};
use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use super::{
    expr::{Expr, inline_expr_parser},
    item::Item,
    local::{Local, parse_local},
};

#[derive(Debug, Clone)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    /// A local (let) binding.
    Let(Local),
    /// An item definition.
    Item(Item),
    /// Expr without trailing semi-colon.
    Expr(Expr),
    /// Expr with a trailing semi-colon.
    Semi(Expr),
    /// Just a trailing semi-colon.
    Empty,
}

pub fn parse_stmt<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Stmt, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let let_stmt = parse_local()
        .then_ignore(just(Token::Semicolon))
        .map(StmtKind::Let);

    let expr_stmt = inline_expr_parser()
        .map(StmtKind::Expr);

    let_stmt.or(expr_stmt)
        .map_with(|kind, e| Stmt {
            id: NodeId::new(),
            kind,
            span: e.span(),
        })
}
