use crate::{Span, ast::NodeId};
use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use super::{
    expr::{Expr, inline_expr_parser},
    item::Item,
    local::{Local, parse_local},
};

/// A statement. Following Rust AST structure exactly.
/// 
/// No `attrs` or `tokens` fields because each `StmtKind` variant
/// contains an AST node with those fields. (Except for `StmtKind::Empty`,
/// which never has attrs or tokens)
#[derive(Debug, Clone)]
pub struct Stmt {
    /// Unique identifier for this statement node
    pub id: NodeId,
    /// The specific kind of statement
    pub kind: StmtKind,
    /// Source location span for this statement
    pub span: Span,
}

/// Statement kinds, following Rust AST enum structure exactly.
/// This is a subset of the full Rust StmtKind enum.
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// A local (let) binding (e.g., `let x = 5;`).
    Let(Box<Local>),
    
    /// An item definition (e.g., function, struct, etc.).
    Item(Box<Item>),
    
    /// An expression without trailing semicolon.
    /// The expression is evaluated and its value is used.
    Expr(Box<Expr>),
    
    /// An expression with a trailing semicolon.
    /// The expression is evaluated but its value is discarded.
    Semi(Box<Expr>),
    
    /// Just a trailing semicolon (`;`).
    /// This is a no-op statement.
    Empty,
    
    // Note: We could add MacCall(Box<MacCallStmt>) here for macro calls in statements
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
