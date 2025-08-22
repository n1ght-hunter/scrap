//! Function call expressions
//! 
//! This module handles parsing of function calls with arguments.
//! Function calls have the highest precedence among binary operations.

use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, utils::LocalVec};
use super::{Expr, ExprKind};

/// Parse function call expressions
/// Takes a base parser for the function expression and an argument parser
pub fn call_parser<'tokens, 'src: 'tokens, I>(
    base_parser: impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone,
    arg_parser: impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
) -> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    base_parser.foldl_with(
        arg_parser
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<LocalVec<_>>()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .repeated(),
        |f, args, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Call(Box::new(f), args),
            span: e.span(),
        },
    )
}

/// Parse function call expressions with inline arguments
/// This version uses the same parser for both function and arguments (recursive)
pub fn call_parser_recursive<'tokens, 'src: 'tokens, I>(
    atom_parser: impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone,
    inline_expr_parser: impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
) -> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    atom_parser.foldl_with(
        inline_expr_parser
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<LocalVec<_>>()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .repeated(),
        |f, args, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Call(Box::new(f), args),
            span: e.span(),
        },
    )
}
