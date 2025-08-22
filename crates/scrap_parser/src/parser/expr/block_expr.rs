//! Block expressions
//! 
//! This module handles parsing of block expressions enclosed in braces.

use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId};
use super::{Expr, ExprKind};

/// Parse block expressions
pub fn block_expr_parser<'tokens, 'src: 'tokens, I>(
    expr_parser: impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
) -> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    expr_parser
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .recover_with(via_parser(nested_delimiters(
            Token::LBrace,
            Token::RBrace,
            [
                (Token::LParen, Token::RParen),
                (Token::LBracket, Token::RBracket),
            ],
            |span| Expr {
                id: NodeId::new(),
                kind: ExprKind::Error,
                span,
            },
        )))
        .labelled("block expression")
}
