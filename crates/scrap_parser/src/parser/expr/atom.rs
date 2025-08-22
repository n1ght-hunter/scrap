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
