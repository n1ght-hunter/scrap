use crate::{Span, ast::NodeId};
use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use super::{
    expr::{Expr, expr_parser},
    item::{Item, item_parser},
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
    parse_local().map(StmtKind::Let).map_with(|t, e| {
        println!("parse_stmt called with token: {:?}", t);
        Stmt {
            id: NodeId::new(),
            kind: StmtKind::Empty,
            span: e.span(),
        }
    })

    //         .or(item_parser().map(StmtKind::Item))
    //         .or(expr_parser()
    //             .then_ignore(just(Token::Semicolon))
    //             .map(StmtKind::Semi))
    //         .or(expr_parser().map(StmtKind::Expr))
    //         .map_with(|k, e| Stmt {
    //             id: NodeId::new(),
    //             kind: k,
    //             span: e.span(),
    //         })
}
