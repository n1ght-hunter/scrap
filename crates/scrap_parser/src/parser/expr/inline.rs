use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{
    Span,
    ast::NodeId,
    parser::{binary::bin_op_parser, lit::lit_parser, parse_ident, block::Block},
    utils::LocalVec,
};
use super::{Expr, ExprKind};

pub fn expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    recursive(|_expr| {
        // Atomic expressions: literals and identifiers
        let atom = lit_parser()
            .map_with(|lit, e| Expr {
                id: NodeId::new(),
                kind: ExprKind::Lit(lit),
                span: e.span(),
            })
            .or(parse_ident()
                .map_with(|ident, e| Expr {
                    id: NodeId::new(),
                    kind: ExprKind::Path(ident.name),
                    span: e.span(),
                }));

        // Binary expressions: left op right
        let binary_expr = atom.clone()
            .then(bin_op_parser())
            .then(atom.clone())
            .map_with(|((left, op), right), e| Expr {
                id: NodeId::new(),
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
                span: e.span(),
            });

        // For now, we keep a simple structure to avoid recursion issues
        binary_expr.or(atom)
    })
}

pub fn inline_expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let lit = lit_parser()
        .map_with(|lit, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Lit(lit),
            span: e.span(),
        });

    let ident = parse_ident()
        .map_with(|ident, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Path(ident.name),
            span: e.span(),
        });

    // Simple binary expression: ident op lit
    let simple_binary = ident.clone()
        .then(bin_op_parser())
        .then(lit.clone())
        .map_with(|((left, op), right), e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
            span: e.span(),
        });

    // Create a simple block parser that just skips contents
    let simple_block = any()
        .filter(|t| matches!(t, Token::LBrace))
        .ignore_then(
            any()
                .filter(|t| !matches!(t, Token::RBrace))
                .repeated()
        )
        .then_ignore(any().filter(|t| matches!(t, Token::RBrace)))
        .map_with(|_, e| Block {
            stmts: LocalVec::new(),
            id: NodeId::new(), 
            span: e.span(),
        });

    // Simple if expression with simple blocks
    let simple_if = just(Token::If)
        .ignore_then(simple_binary.clone())
        .then(simple_block.clone())
        .then(
            just(Token::Else)
                .ignore_then(simple_block)
                .or_not(),
        )
        .map_with(|((cond, then), else_block_opt), e| Expr {
            id: NodeId::new(),
            kind: ExprKind::If(
                Box::new(cond), 
                Box::new(then), 
                else_block_opt.map(|_block| Box::new(Expr {
                    id: NodeId::new(),
                    kind: ExprKind::Dummy,
                    span: e.span(),
                }))
            ),
            span: e.span(),
        });

    simple_if.or(simple_binary).or(lit).or(ident)
}
