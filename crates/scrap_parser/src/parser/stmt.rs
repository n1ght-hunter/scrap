use crate::{Span, ast::NodeId};
use chumsky::prelude::*;
use scrap_lexer::Token;

use super::{
    ScrapInput, ScrapParser,
    block::Block,
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

pub fn parse_stmt<'tokens, 'src: 'tokens, I>(
    block_parser: impl ScrapParser<'tokens, 'src, I, Block> + 'tokens,
) -> impl ScrapParser<'tokens, 'src, I, Stmt>
where
    I: ScrapInput<'tokens, 'src>,
{
    // Let statements MUST have semicolons
    let let_stmt = parse_local(block_parser.clone())
        .map(|local| StmtKind::Let(Box::new(local)))
        .labelled("let statement");

    // Return statements MUST have semicolons
    let return_stmt = just(Token::Return)
        .ignore_then(inline_expr_parser().or_not())
        .then_ignore(just(Token::Semicolon))
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

    // If statements as expressions
    let if_stmt = just(Token::If)
        .ignore_then(inline_expr_parser())
        .then(block_parser)
        .then_ignore(just(Token::Semicolon).or_not())
        .map_with(
            |(cond, then_block),
             e: &mut chumsky::input::MapExtra<
                '_,
                '_,
                I,
                chumsky::extra::Full<
                    crate::parse_error::ParseError<'_, Token<'_>>,
                    super::State,
                    (),
                >,
            >| {
                let if_expr = crate::parser::expr::Expr {
                    id: e.state().new_node_id(),
                    kind: crate::parser::expr::ExprKind::If(
                        Box::new(cond),
                        Box::new(then_block),
                        None,
                    ),
                    span: e.span(),
                };
                StmtKind::Semi(Box::new(if_expr))
            },
        )
        .labelled("if statement")
        .as_context();

    // Expressions with semicolons (discarded values)
    let expr_with_semi = inline_expr_parser()
        .then_ignore(just(Token::Semicolon))
        .map(|expr| StmtKind::Semi(Box::new(expr)))
        .labelled("expression statement")
        .as_context();

    // Try statements in order - put let_stmt first so it doesn't get consumed as expr_without_semi
    choice((
        let_stmt,
        return_stmt,
        if_stmt,
        expr_with_semi,
        inline_expr_parser().map(|expr| StmtKind::Expr(Box::new(expr))),
    ))
    .map_with(
        |kind,
         e: &mut chumsky::input::MapExtra<
            '_,
            '_,
            I,
            chumsky::extra::Full<crate::parse_error::ParseError<'_, Token<'_>>, super::State, ()>,
        >| Stmt {
            id: e.state().new_node_id(),
            kind,
            span: e.span(),
        },
    )
    .labelled("statement")
    .recover_with(skip_then_retry_until(
        any().ignored(),
        one_of([Token::Semicolon, Token::RBrace, Token::LBrace]).ignored(),
    ))
}
