//! Expression parsing following the Rust parser structure

use super::{
    ScrapInput, ScrapParser,
    block::{Block, block_parser},
    ident::parse_ident,
    lit::{Lit, lit_parser},
    operators::{AssignOpKind, BinOp, BinOpKind},
};
use crate::{Spanned, ast::NodeId, utils::LocalVec};
use chumsky::prelude::*;
use scrap_lexer::Token;

/// An expression node in the AST
#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: crate::Span,
}

/// Expression kinds - subset of Rust's ExprKind enum
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// An array literal (e.g., `[a, b, c, d]`)
    Array(LocalVec<Box<Expr>>),
    /// A function call
    Call(Box<Expr>, LocalVec<Box<Expr>>),
    /// A binary operation (e.g., `a + b`, `a * b`)
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// A literal value (e.g., `1`, `"foo"`)
    Lit(Lit),
    /// An `if` block, with an optional `else` block
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    /// A block (`{ ... }`)
    Block(Box<Block>),
    /// Variable reference
    Path(String),
    /// A parenthesized expression
    Paren(Box<Expr>),
    /// A `return` expression
    Return(Option<Box<Expr>>),
    /// An assignment (`place = expr`)
    Assign(Box<Expr>, Box<Expr>),
    /// An assignment with an operator (`place += expr`)
    AssignOp(Spanned<AssignOpKind>, Box<Expr>, Box<Expr>),
    /// Error placeholder
    Err,
}

/// Parse a full expression with all operators and precedence
pub fn expr_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    recursive(|expr| {
        // Atom expressions (highest precedence)
        let atom = choice((
            // Literals
            lit_parser().map_with(|lit, e| Expr {
                id: e.state().new_node_id(),
                kind: ExprKind::Lit(lit),
                span: e.span(),
            }),
            // Identifiers/paths
            parse_ident().map_with(|ident, e| Expr {
                id: e.state().new_node_id(),
                kind: ExprKind::Path(ident.name),
                span: e.span(),
            }),
            // Parenthesized expressions
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map_with(
                    |inner_expr,
                     e: &mut chumsky::input::MapExtra<
                        '_,
                        '_,
                        I,
                        extra::Full<
                            crate::parse_error::ParseError<'_, Token<'_>>,
                            super::State,
                            (),
                        >,
                    >| Expr {
                        id: e.state().new_node_id(),
                        kind: ExprKind::Paren(Box::new(inner_expr)),
                        span: e.span(),
                    },
                )
                .boxed(),
            // Blocks
            block_parser()
                .map_with(
                    |block,
                     e: &mut chumsky::input::MapExtra<
                        '_,
                        '_,
                        I,
                        extra::Full<
                            crate::parse_error::ParseError<'_, Token<'_>>,
                            super::State,
                            (),
                        >,
                    >| Expr {
                        id: e.state().new_node_id(),
                        kind: ExprKind::Block(Box::new(block)),
                        span: e.span(),
                    },
                )
                .boxed(),
            // Arrays - simplified to avoid nested map_with
            expr.clone()
                .map(Box::new)
                .separated_by(just(Token::Comma))
                .collect::<LocalVec<_>>()
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .map_with(
                    |elements,
                     e: &mut chumsky::input::MapExtra<
                        '_,
                        '_,
                        I,
                        extra::Full<
                            crate::parse_error::ParseError<'_, Token<'_>>,
                            super::State,
                            (),
                        >,
                    >| Expr {
                        id: e.state().new_node_id(),
                        kind: ExprKind::Array(elements),
                        span: e.span(),
                    },
                )
                .boxed(),
            // Return expressions
            just(Token::Return)
                .ignore_then(expr.clone().map(Box::new).or_not())
                .map_with(|return_expr, e| Expr {
                    id: e.state().new_node_id(),
                    kind: ExprKind::Return(return_expr),
                    span: e.span(),
                })
                .boxed(),
        ))
        .boxed();

        // Function calls (postfix)
        let call = atom
            .foldl_with(
                expr.clone()
                    .map(Box::new)
                    .separated_by(just(Token::Comma))
                    .collect::<LocalVec<_>>()
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .repeated(),
                |func, args, e| Expr {
                    id: e.state().new_node_id(),
                    kind: ExprKind::Call(Box::new(func), args),
                    span: e.span(),
                },
            )
            .boxed();

        // Multiplication and division
        let product = call
            .clone()
            .foldl_with(
                choice((
                    just(Token::Mul).to(BinOpKind::Mul),
                    just(Token::Div).to(BinOpKind::Div),
                    just(Token::Rem).to(BinOpKind::Rem),
                ))
                .then(call)
                .repeated(),
                |lhs, (op, rhs), e| Expr {
                    id: e.state().new_node_id(),
                    kind: ExprKind::Binary(
                        Spanned {
                            node: op,
                            span: e.span(),
                        },
                        Box::new(lhs),
                        Box::new(rhs),
                    ),
                    span: e.span(),
                },
            )
            .boxed();

        // Addition and subtraction
        let sum = product
            .clone()
            .foldl_with(
                choice((
                    just(Token::Add).to(BinOpKind::Add),
                    just(Token::Sub).to(BinOpKind::Sub),
                ))
                .then(product)
                .repeated(),
                |lhs, (op, rhs), e| Expr {
                    id: e.state().new_node_id(),
                    kind: ExprKind::Binary(
                        Spanned {
                            node: op,
                            span: e.span(),
                        },
                        Box::new(lhs),
                        Box::new(rhs),
                    ),
                    span: e.span(),
                },
            )
            .boxed();

        // Comparisons
        let comparison = sum
            .clone()
            .foldl_with(
                choice((
                    just(Token::Eq).to(BinOpKind::Eq),
                    just(Token::Ne).to(BinOpKind::Ne),
                    just(Token::Lt).to(BinOpKind::Lt),
                    just(Token::Le).to(BinOpKind::Le),
                    just(Token::Gt).to(BinOpKind::Gt),
                    just(Token::Ge).to(BinOpKind::Ge),
                ))
                .then(sum)
                .repeated(),
                |lhs, (op, rhs), e| Expr {
                    id: e.state().new_node_id(),
                    kind: ExprKind::Binary(
                        Spanned {
                            node: op,
                            span: e.span(),
                        },
                        Box::new(lhs),
                        Box::new(rhs),
                    ),
                    span: e.span(),
                },
            )
            .boxed();

        // Logical AND
        let logical_and = comparison
            .clone()
            .foldl_with(
                just(Token::And)
                    .to(BinOpKind::And)
                    .then(comparison)
                    .repeated(),
                |lhs, (op, rhs), e| Expr {
                    id: e.state().new_node_id(),
                    kind: ExprKind::Binary(
                        Spanned {
                            node: op,
                            span: e.span(),
                        },
                        Box::new(lhs),
                        Box::new(rhs),
                    ),
                    span: e.span(),
                },
            )
            .boxed();

        // Logical OR
        let logical_or = logical_and
            .clone()
            .foldl_with(
                just(Token::Or)
                    .to(BinOpKind::Or)
                    .then(logical_and)
                    .repeated(),
                |lhs, (op, rhs), e| Expr {
                    id: e.state().new_node_id(),
                    kind: ExprKind::Binary(
                        Spanned {
                            node: op,
                            span: e.span(),
                        },
                        Box::new(lhs),
                        Box::new(rhs),
                    ),
                    span: e.span(),
                },
            )
            .boxed();

        // Assignments (lowest precedence)
        logical_or
            .clone()
            .foldl_with(
                choice((
                    just(Token::Assign).to(None),
                    choice((
                        just(Token::AddAssign).to(AssignOpKind::AddAssign),
                        just(Token::SubAssign).to(AssignOpKind::SubAssign),
                        just(Token::MulAssign).to(AssignOpKind::MulAssign),
                        just(Token::DivAssign).to(AssignOpKind::DivAssign),
                        just(Token::RemAssign).to(AssignOpKind::RemAssign),
                    ))
                    .map(Some),
                ))
                .then(logical_or)
                .repeated(),
                |lhs, (assign_op, rhs), e| match assign_op {
                    None => Expr {
                        id: e.state().new_node_id(),
                        kind: ExprKind::Assign(Box::new(lhs), Box::new(rhs)),
                        span: e.span(),
                    },
                    Some(op) => Expr {
                        id: e.state().new_node_id(),
                        kind: ExprKind::AssignOp(
                            Spanned {
                                node: op,
                                span: e.span(),
                            },
                            Box::new(lhs),
                            Box::new(rhs),
                        ),
                        span: e.span(),
                    },
                },
            )
            .boxed()
    })
    .labelled("expression")
}

/// Parse inline expressions (simplified version for simple cases)
pub fn inline_expr_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    expr_parser().labelled("inline expression").boxed()
}
