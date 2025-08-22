use crate::{Span, ast::NodeId};
use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use super::{
    expr::{Expr, inline_expr_parser},
    item::Item,
    local::{Local, parse_local},
};

/// A statement. Following Rust AST structure.
#[derive(Debug, Clone)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
    // Note: In Rust AST, attrs and tokens are in the StmtKind variants, not here
    // but we could add them here if needed for our simplified version
}

/// Statement kinds, following Rust AST enum structure
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// A local (let) binding.
    Let(Box<Local>),
    /// An item definition.
    Item(Box<Item>),
    /// Expr without trailing semi-colon.
    Expr(Box<Expr>),
    /// Expr with a trailing semi-colon.
    Semi(Box<Expr>),
    /// Just a trailing semi-colon.
    Empty,
    // Note: We could add MacCall(Box<MacCallStmt>) here in the future
}

pub fn parse_stmt<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Stmt, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let let_stmt = parse_local()
        .then_ignore(just(Token::Semicolon))
        .map(|local| StmtKind::Let(Box::new(local)));

    let expr_stmt = inline_expr_parser()
        .map(|expr| StmtKind::Expr(Box::new(expr)));

    let_stmt.or(expr_stmt)
        .map_with(|kind, e| Stmt {
            id: NodeId::new(),
            kind,
            span: e.span(),
        })
}
