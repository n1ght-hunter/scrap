use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;
use thin_vec::ThinVec;

use crate::{
    Span,
    ast::NodeId,
    parser::{binary::bin_op_parser, lit::lit_parser},
    utils::LocalVec,
};

use super::{
    binary::BinOp,
    block::{Block, block_parser},
    lit::Lit,
    local::parse_local,
    parse_ident,
};

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Dummy,
    Path(String),
    Call(Box<Expr>, LocalVec<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Lit(Lit),
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
}

pub fn expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    recursive(|expr| {
        let inline_expr = inline_expr_parser();

        inline_expr
    })
}

pub fn inline_expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    recursive(|inline_expr| {
        let lit = lit_parser()
            .map_with(|lit, e| Expr {
                id: NodeId::new(),
                kind: ExprKind::Lit(lit),
                span: e.span(),
            })
            .labelled("literal");

        let ident = parse_ident();

        let atom = atom_parser();

        ident
    })
}

pub fn items_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, LocalVec<Expr>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    expr_parser()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<LocalVec<_>>()
}

/// 'Atoms' are expressions that contain no ambiguity
pub fn atom_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let lit = lit_parser()
        .map_with(|lit, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Lit(lit),
            span: e.span(),
        })
        .labelled("literal");

    let ident = parse_ident();

    let items = expr_parser()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<LocalVec<_>>();

    items
}

pub fn call_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    atom_parser().foldl_with(
        items_parser()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .repeated(),
        |f, args, e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Call(Box::new(f), args),
            span: e.span(),
        },
    )
}

pub fn lit_or_path_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    lit_parser().map(ExprKind::Lit).or(path_expr_parser())
}

pub fn path_expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    select! {
        Token::Ident(ident) => ident,
    }
    .map(|p| ExprKind::Path(p.to_string()))
}

pub fn binary_expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    lit_or_path_parser()
        .map_with(|expr, e| Expr {
            id: NodeId::new(),
            kind: expr,
            span: e.span(),
        })
        .then(bin_op_parser())
        .then(lit_or_path_parser().map_with(|expr, e| Expr {
            id: NodeId::new(),
            kind: expr,
            span: e.span(),
        }))
        .map(|((left, op), right)| ExprKind::Binary(op, Box::new(left), Box::new(right)))
}

pub fn if_expr_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    recursive(|if_| {
        just(Token::If)
            .ignore_then(expr_parser())
            .then(block_parser())
            .then(
                just(Token::Else)
                    .ignore_then(expr_parser().or(if_))
                    .or_not(),
            )
            .map_with(|((cond, then), else_opt), e| Expr {
                id: NodeId::new(),
                kind: ExprKind::If(Box::new(cond), Box::new(then), else_opt.map(Box::new)),
                span: e.span(),
            })
    })
}

pub fn dummy_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    just(Token::If).not().map_with(|_, e| Expr {
        id: NodeId::new(),
        kind: ExprKind::Dummy,
        span: e.span(),
    })
}
