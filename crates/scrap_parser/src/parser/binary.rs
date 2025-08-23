//! Binary operations and precedence handling
//!
//! This module contains binary operator definitions and parsers that handle
//! proper operator precedence according to mathematical conventions.
//! The definitions follow the Rust AST structure exactly.

use crate::Spanned;

use chumsky::prelude::*;
use scrap_lexer::Token;

use super::{ScrapInput, ScrapParser};

/// A binary operator with its source location span.
/// This matches the Rust AST pattern of wrapping operator kinds with span information.
pub type BinOp = Spanned<BinOpKind>;

/// Binary operator kinds, following Rust AST enum structure exactly.
/// These represent the different types of binary operations available in the language.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinOpKind {
    /// The `+` operator (addition)
    Add,
    /// The `-` operator (subtraction)
    Sub,
    /// The `*` operator (multiplication)
    Mul,
    /// The `/` operator (division)
    Div,
    /// The `%` operator (modulus)
    Rem,
    /// The `&&` operator (logical and)
    And,
    /// The `||` operator (logical or)
    Or,
    /// The `^` operator (bitwise xor)
    BitXor,
    /// The `&` operator (bitwise and)
    BitAnd,
    /// The `|` operator (bitwise or)
    BitOr,
    /// The `<<` operator (shift left)
    Shl,
    /// The `>>` operator (shift right)
    Shr,
    /// The `==` operator (equality)
    Eq,
    /// The `<` operator (less than)
    Lt,
    /// The `<=` operator (less than or equal to)
    Le,
    /// The `!=` operator (not equal to)
    Ne,
    /// The `>=` operator (greater than or equal to)
    Ge,
    /// The `>` operator (greater than)
    Gt,
}

/// Basic binary operator parser
pub fn bin_op_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, BinOp>
where
    I: ScrapInput<'tokens, 'src>,
{
    let op = choice((
        just(Token::Plus).to(BinOpKind::Add),
        just(Token::Minus).to(BinOpKind::Sub),
        just(Token::Star).to(BinOpKind::Mul),
        just(Token::Slash).to(BinOpKind::Div),
        just(Token::Percent).to(BinOpKind::Rem),
        just(Token::And).to(BinOpKind::And),
        just(Token::Or).to(BinOpKind::Or),
        just(Token::BitXor).to(BinOpKind::BitXor),
        just(Token::BitAnd).to(BinOpKind::BitAnd),
        just(Token::BitOr).to(BinOpKind::BitOr),
        just(Token::Shl).to(BinOpKind::Shl),
        just(Token::Shr).to(BinOpKind::Shr),
        just(Token::Eq).to(BinOpKind::Eq),
        just(Token::Lt).to(BinOpKind::Lt),
        just(Token::Le).to(BinOpKind::Le),
        just(Token::Ne).to(BinOpKind::Ne),
        just(Token::Ge).to(BinOpKind::Ge),
        just(Token::Gt).to(BinOpKind::Gt),
    ));

    op.map_with(|kind, e| Spanned {
        node: kind,
        span: e.span(),
    })
}

// Forward declaration - we'll import Expr where needed
use crate::parser::expr::{Expr, ExprKind};

/// Parse multiplication and division operations (highest precedence binary ops)
pub fn product_parser<'tokens, 'src: 'tokens, I>(
    base_parser: impl ScrapParser<'tokens, 'src, I, Expr>,
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    let mul_div_ops = just(Token::Star).or(just(Token::Slash));

    base_parser.clone().foldl_with(
        mul_div_ops.then(base_parser).repeated(),
        |lhs, (op_token, rhs), e| {
            let op = Spanned {
                node: match op_token {
                    Token::Star => BinOpKind::Mul,
                    Token::Slash => BinOpKind::Div,
                    _ => unreachable!(),
                },
                span: e.span(),
            };
            Expr {
                id: e.state().new_node_id(),
                kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)),
                span: e.span(),
            }
        },
    )
}

/// Parse addition and subtraction operations (medium precedence)
pub fn sum_parser<'tokens, 'src: 'tokens, I>(
    base_parser: impl ScrapParser<'tokens, 'src, I, Expr>,
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    let add_sub_ops = just(Token::Plus).or(just(Token::Minus));

    base_parser.clone().foldl_with(
        add_sub_ops.then(base_parser).repeated(),
        |lhs, (op_token, rhs), e| {
            let op = Spanned {
                node: match op_token {
                    Token::Plus => BinOpKind::Add,
                    Token::Minus => BinOpKind::Sub,
                    _ => unreachable!(),
                },
                span: e.span(),
            };
            Expr {
                id: e.state().new_node_id(),
                kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)),
                span: e.span(),
            }
        },
    )
}

/// Parse comparison operations (lowest precedence)
pub fn comparison_parser<'tokens, 'src: 'tokens, I>(
    base_parser: impl ScrapParser<'tokens, 'src, I, Expr>,
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    let comparison_ops = just(Token::Gt)
        .or(just(Token::Lt))
        .or(just(Token::Ge))
        .or(just(Token::Le))
        .or(just(Token::Eq))
        .or(just(Token::Ne));

    base_parser.clone().foldl_with(
        comparison_ops.then(base_parser).repeated(),
        |lhs, (op_token, rhs), e| {
            let op = Spanned {
                node: match op_token {
                    Token::Gt => BinOpKind::Gt,
                    Token::Lt => BinOpKind::Lt,
                    Token::Ge => BinOpKind::Ge,
                    Token::Le => BinOpKind::Le,
                    Token::Eq => BinOpKind::Eq,
                    Token::Ne => BinOpKind::Ne,
                    _ => unreachable!(),
                },
                span: e.span(),
            };
            Expr {
                id: e.state().new_node_id(),
                kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)),
                span: e.span(),
            }
        },
    )
}
