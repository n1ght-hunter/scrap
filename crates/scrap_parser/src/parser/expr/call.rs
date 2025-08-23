//! Function call expressions
//!
//! This module handles parsing of function calls with arguments.
//! Function calls have the highest precedence among binary operations.

use chumsky::prelude::*;
use scrap_lexer::Token;

use super::{Expr, ExprKind};
use crate::parser::{ScrapInput, ScrapParser};
use crate::{ast::NodeId, utils::LocalVec};

/// Parse function call expressions
pub fn call_parser<'tokens, 'src: 'tokens, I>(
    base_parser: impl ScrapParser<'tokens, 'src, I, Expr>,
    arg_parser: impl ScrapParser<'tokens, 'src, I, Expr>,
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    base_parser.foldl_with(
        arg_parser
            .map(Box::new)
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
