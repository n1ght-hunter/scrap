//! Expression parsing following the Rust parser structure

use super::{
    ScrapInput, ScrapParser,
    block::Block,
    ident::Ident,
    lit::{Lit, lit_parser},
    operators::{AssignOpKind, BinOp, BinOpKind},
    parse_ident,
};
use crate::{Spanned, ast::NodeId, parser::operators::ops_parser, utils::LocalVec};
use chumsky::prelude::*;
use scrap_lexer::Token;
use thin_vec::ThinVec;

#[derive(Debug, Clone)]
pub struct PathSegment {
    /// The identifier portion of this path segment.
    pub ident: Ident,

    pub id: NodeId,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub span: crate::Span,
    /// The segments in the path: the things separated by `::`.
    /// Global paths begin with `kw::PathRoot`.
    pub segments: ThinVec<PathSegment>,
}

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
    Path(Path),
    /// A parenthesized expression
    Paren(Box<Expr>),
    /// A `return` expression
    Return(Option<Box<Expr>>),
    /// An assignment (`place = expr`)
    Assign(Box<Expr>, Box<Expr>, crate::Span),
    /// An assignment with an operator (`place += expr`)
    AssignOp(Spanned<AssignOpKind>, Box<Expr>, Box<Expr>),
    /// Error placeholder
    Err,
}

/// Parse a full expression with all operators and precedence
pub fn expr_parser<'tokens, 'src: 'tokens, I>(
    block_parser: impl ScrapParser<'tokens, 'src, I, Block> + 'tokens,
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    recursive(|expr| {
        choice((
            just(Token::Return)
                .then(inline_expr_parser().or_not())
                .then_ignore(just(Token::Semicolon))
                .map_with(|(_, expr), e| Expr {
                    id: e.state().new_node_id(),
                    kind: ExprKind::Return(expr.map(Box::new)),
                    span: e.span(),
                }),
            lit_parser().map_with(|lit, e| Expr {
                id: e.state().new_node_id(),
                kind: ExprKind::Lit(lit),
                span: e.span(),
            }),
            just(Token::If)
                .then(inline_expr_parser())
                .then(block_parser)
                .then(just(Token::Else).ignore_then(expr.clone()).or_not())
                .map_with(|(((_, cond), then_block), else_block), e| Expr {
                    id: e.state().new_node_id(),
                    kind: ExprKind::If(
                        Box::new(cond),
                        Box::new(then_block),
                        else_block.map(Box::new),
                    ),
                    span: e.span(),
                }),
            // just(Token::If)
            //     .then(inline_expr_parser())
            //     .then(block_parser())
            //     .then(just(Token::Else).ignore_then(block_parser()).or_not())
            //     .map_with(|(((_, cond), then_block), else_block), e| Expr {
            //         id: e.state().new_node_id(),
            //         kind: ExprKind::If(
            //             Box::new(cond),
            //             Box::new(then_block),
            //             else_block.map(Box::new),
            //         ),
            //         span: e.span(),
            //     }),
        ))
    })
    .labelled("expression")
}

/// Parse inline expressions (simplified version for simple cases)
pub fn inline_expr_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    atom_parser()
}

pub fn atom_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    let lit_parser = lit_parser().map_with(|lit, e| Expr {
        id: e.state().new_node_id(),
        kind: ExprKind::Lit(lit),
        span: e.span(),
    });

    let ident = parse_ident().map_with(|ident, e| Expr {
        id: e.state().new_node_id(),
        kind: ExprKind::Path(Path {
            span: e.span(),
            segments: ThinVec::from(vec![PathSegment {
                ident,
                id: e.state().new_node_id(),
            }]),
        }),
        span: e.span(),
    });

    ops_parser(choice((lit_parser, ident)))
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        State,
        block::block_parser,
        lit::{LitKind, TempLit},
    };

    use super::*;

    #[test]
    fn test_parse_simple_return() {
        let input = [Token::Return, Token::Int(42), Token::Semicolon];
        let mut state = State::new("test.sc");
        let expr = expr_parser(block_parser())
            .parse_with_state(&input, &mut state)
            .unwrap();

        match expr.kind {
            ExprKind::Return(Some(boxed_expr)) => match boxed_expr.kind {
                ExprKind::Lit(lit) => match lit.temp_lit {
                    TempLit::Int(value) => assert_eq!(value, 42),
                    _ => panic!("Expected integer literal"),
                },
                _ => panic!("Expected literal expression"),
            },
            _ => panic!("Expected return expression"),
        }
    }
}
