//! Atomic expressions
//! 
//! This module handles the basic building blocks of expressions:
//! literals, identifiers, paths, and parenthesized expressions.
//! These are the fundamental atomic units that cannot be decomposed further.

use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, utils::LocalVec};
use super::{Expr, ExprKind};
use crate::parser::{lit::lit_parser, parse_ident, ScrapParser, ScrapInput};

/// Parse atomic literals
pub fn literal_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
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
pub fn identifier_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    parse_ident()
        .map_with(|ident, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Path(ident.name),
            span: e.span(),
        })
        .labelled("identifier")
}

/// Parse array expressions
pub fn array_parser<'tokens, 'src: 'tokens, I>(
    expr_parser: impl ScrapParser<'tokens, 'src, I, Expr>
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    expr_parser
        .map(Box::new)
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<LocalVec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .map_with(|elements, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Array(elements),
            span: e.span(),
        })
        .labelled("array")
}

/// Parse parenthesized expressions (creates Paren variant)
pub fn parenthesized_parser<'tokens, 'src: 'tokens, I>(
    expr_parser: impl ScrapParser<'tokens, 'src, I, Expr>
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    expr_parser
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .map_with(|expr, e| Expr {
            id: NodeId::new(), 
            kind: ExprKind::Paren(Box::new(expr)),
            span: e.span(),
        })
        .labelled("parenthesized expression")
}

/// Parse return expressions
pub fn return_parser<'tokens, 'src: 'tokens, I>(
    expr_parser: impl ScrapParser<'tokens, 'src, I, Expr>
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    just(Token::Return)
        .ignore_then(expr_parser.or_not())
        .map_with(|expr, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Return(expr.map(Box::new)),
            span: e.span(),
        })
        .labelled("return expression")
}

/// Parse atomic expressions with error recovery
pub fn atom_with_recovery<'tokens, 'src: 'tokens, I>(
    expr_parser: impl ScrapParser<'tokens, 'src, I, Expr>
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    choice((
        literal_parser(),
        identifier_parser(),
        array_parser(expr_parser.clone()),
        parenthesized_parser(expr_parser),
    ))
    .recover_with(via_parser(nested_delimiters(
        Token::LParen,
        Token::RParen,
        [
            (Token::LBrace, Token::RBrace),
            (Token::LBracket, Token::RBracket),
        ],
        |span| Expr {
            id: NodeId::new(),
            kind: ExprKind::Err,
            span,
        },
    )))
}
