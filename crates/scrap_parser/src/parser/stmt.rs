use crate::{Span, ast::NodeId, utils::LocalVec};
use chumsky::prelude::*;
use scrap_lexer::Token;

use super::{
    expr::{Expr, inline_expr_parser},
    item::Item,
    local::{Local, parse_local},
    ScrapParser, ScrapInput,
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

/// Helper parser for better error messages when semicolons are expected
fn expect_semicolon<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, ()>
where
    I: ScrapInput<'tokens, 'src>,
{
    just(Token::Semicolon)
        .ignored()
        .labelled("semicolon")
        .as_context()
}

pub fn parse_stmt<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Stmt>
where
    I: ScrapInput<'tokens, 'src>,
{
    // Let statements MUST have semicolons
    let let_stmt = parse_local()
        .then_ignore(expect_semicolon())
        .map(|local| StmtKind::Let(Box::new(local)))
        .labelled("let statement");

    // Return statements MUST have semicolons
    let return_stmt = just(Token::Return)
        .ignore_then(inline_expr_parser().or_not())
        .then_ignore(expect_semicolon())
        .map_with(|expr, e| {
            let return_expr = crate::parser::expr::Expr::new(
                crate::parser::expr::ExprKind::Return(expr.map(Box::new)),
                e.span()
            );
            StmtKind::Semi(Box::new(return_expr))
        })
        .labelled("return statement")
        .as_context();

    // Empty statements (just semicolons)
    let empty_stmt = just(Token::Semicolon)
        .map(|_| StmtKind::Empty)
        .labelled("empty statement");

    // Expressions with semicolons (discarded values)
    let expr_with_semi = {
        // Create a parser that includes function calls for semicolon-terminated expressions
        let expr_parser = recursive(|_expr| {
            let basic_expr = inline_expr_parser();
            
            // Add function call support for semicolon-terminated statements
            let atom = basic_expr.clone();
            let call_expr = atom.clone()
                .then(
                    atom.clone()
                        .map(Box::new)
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .collect::<LocalVec<_>>()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .or_not()
                )
                .map_with(|(f, args_opt), e| {
                    match args_opt {
                        Some(args) => crate::parser::expr::Expr {
                            id: NodeId::new(),
                            kind: crate::parser::expr::ExprKind::Call(Box::new(f), args),
                            span: e.span(),
                        },
                        None => f,
                    }
                });
            
            call_expr.or(basic_expr)
        });
        
        expr_parser
            .then_ignore(expect_semicolon())
            .map(|expr| StmtKind::Semi(Box::new(expr)))
            .labelled("expression statement")
            .as_context()
    };

    // Expressions without semicolons - only valid as last statement in a block
    // This should have lowest priority to avoid consuming let statements
    let expr_without_semi = inline_expr_parser()
        .map(|expr| StmtKind::Expr(Box::new(expr)))
        .labelled("tail expression");

    // Try statements in order - put let_stmt first so it doesn't get consumed as expr_without_semi
    choice((
        let_stmt,
        return_stmt,
        empty_stmt,
        expr_with_semi,
        expr_without_semi,
    ))
    .map_with(|kind, e| Stmt {
        id: NodeId::new(),
        kind,
        span: e.span(),
    })
    .labelled("statement")
    .recover_with(skip_then_retry_until(
        any().ignored(),
        one_of([Token::Semicolon, Token::RBrace, Token::LBrace]).ignored(),
    ))
}
