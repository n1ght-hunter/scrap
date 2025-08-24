use crate::{Span, ast::NodeId};
use chumsky::prelude::*;
use scrap_lexer::Token;

use super::{
    ScrapInput, ScrapParser,
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
    /// A local (let) binding (e.g., `let <pat>:<ty> = <expr>;`).
    Let(Box<Local>),

    /// An item definition (e.g., function, struct, etc.).
    Item(Box<Item>),
    /// Expr without trailing semi-colon.
    Expr(Box<Expr>),
    /// Expr with a trailing semi-colon.
    Semi(Box<Expr>),
    /// Just a trailing semi-colon.
    Empty,
}

pub fn parse_stmt<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Stmt>
where
    I: ScrapInput<'tokens, 'src>,
{
    // Let statements MUST have semicolons
    let let_stmt = parse_local()
        .map(|local| StmtKind::Let(Box::new(local)))
        .labelled("let statement");

    // Return statements MUST have semicolons
    let return_stmt = just(Token::Return)
        .ignore_then(inline_expr_parser().or_not())
        .map_with(|expr, e| {
            let return_expr = crate::parser::expr::Expr {
                id: e.state().new_node_id(),
                kind: crate::parser::expr::ExprKind::Return(expr.map(Box::new)),
                span: e.span(),
            };
            StmtKind::Semi(Box::new(return_expr))
        })
        .labelled("return statement")
        .as_context();

    // Expressions with semicolons (discarded values)
    let expr_with_semi = inline_expr_parser()
        .map(|expr| StmtKind::Semi(Box::new(expr)))
        .labelled("expression statement")
        .as_context();

    // Try statements in order - put let_stmt first so it doesn't get consumed as expr_without_semi
    choice((
        let_stmt,
        return_stmt,
        expr_with_semi,
        inline_expr_parser().map(|expr| StmtKind::Expr(Box::new(expr))),
    ))
    .map_with(|kind, e| Stmt {
        id: e.state().new_node_id(),
        kind,
        span: e.span(),
    })
    .labelled("statement")
    .recover_with(skip_then_retry_until(
        any().ignored(),
        one_of([Token::Semicolon, Token::RBrace, Token::LBrace]).ignored(),
    ))
}
