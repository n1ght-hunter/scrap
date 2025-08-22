//! Atomic expressions
//! 
//! This module handles the basic building blocks of expressions:
//! literals, identifiers, paths, and parenthesized expressions.
//! These are the fundamental atomic units that cannot be decomposed further.

use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, parser::lit::lit_parser};
use super::{Expr, ExprKind};
use crate::parser::parse_ident;

/// Parse atomic literals
pub fn literal_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    lit_parser()
        .map_with(|lit, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Lit(lit),
            span: e.span(),
        })
        .labelled("literal")
}

/// Parse identifier expressions
pub fn identifier_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parse_ident()
        .map_with(|ident, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Path(ident.name),
            span: e.span(),
        })
        .labelled("identifier")
}

/// Parse parenthesized expressions
pub fn parenthesized_parser<'tokens, 'src: 'tokens, I>(
    expr_parser: impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
) -> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    expr_parser
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .labelled("parenthesized expression")
}

/// Parse atomic expressions with error recovery
pub fn atom_with_recovery<'tokens, 'src: 'tokens, I>(
    paren_expr: impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
) -> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    literal_parser()
        .or(identifier_parser())
        .or(paren_expr)
        .recover_with(via_parser(nested_delimiters(
            Token::LParen,
            Token::RParen,
            [
                (Token::LBrace, Token::RBrace),
                (Token::LBracket, Token::RBracket),
            ],
            |span| Expr {
                id: NodeId::new(),
                kind: ExprKind::Error,
                span,
            },
        )))
}

/// Basic atom parser (literals and identifiers only)
pub fn atom_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    literal_parser().or(identifier_parser())
}

/// Parse path expressions (direct identifier parsing)
/// 
/// This is essentially the same as identifier_parser but uses direct token selection.
/// It can be used as an alternative when you need direct path parsing.
pub fn path_expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    select! {
        Token::Ident(ident) => ident,
    }
    .map_with(|p, e| Expr {
        id: NodeId::new(),
        kind: ExprKind::Path(p.to_string()),
        span: e.span(),
    })
}

/// Parse literal or path expressions
/// 
/// This parser accepts either a literal value or a path/identifier expression.
pub fn lit_or_path_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    literal_parser().or(path_expr_parser())
}
