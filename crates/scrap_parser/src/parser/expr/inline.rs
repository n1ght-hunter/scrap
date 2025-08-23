//! Inline expression parsing coordination
//! 
//! This module coordinates all the different expression parsers from their respective modules.

use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, utils::LocalVec};
use super::{
    Expr, ExprKind,
    atom::{parenthesized_parser, atom_with_recovery},
    call::call_parser,
    block_expr::block_expr_parser,
    if_expr::if_expr_parser,
};
use crate::parser::binary::{product_parser, sum_parser, comparison_parser, bin_op_parser};
use crate::parser::{lit::lit_parser, parse_ident, ScrapParser, ScrapInput};

/// Main expression parser with full precedence handling
/// 
/// This parser coordinates all the modular expression parsers with proper operator precedence.
pub fn expr_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    recursive(|_expr| {
        let inline_expr = recursive(|inline_expr| {
            // ===== ATOMIC EXPRESSIONS =====
            let paren_expr = parenthesized_parser::<I>(inline_expr.clone());
            
            // Combine atoms with error recovery
            let atom = atom_with_recovery(paren_expr);

            // ===== FUNCTION CALLS =====
            let call = call_parser(atom, inline_expr.clone());

            // ===== BINARY OPERATIONS WITH PRECEDENCE =====
            // Apply operator precedence: multiplication/division -> addition/subtraction -> comparison
            let product = product_parser(call);
            let sum = sum_parser(product);
            let compare = comparison_parser(sum);

            compare.labelled("expression").as_context()
        });

        // ===== CONTROL FLOW EXPRESSIONS =====
        let block = block_expr_parser();
        let if_expr = if_expr_parser(inline_expr.clone());

        // Combine all expression types
        let block_expr = block.or(if_expr);

        block_expr
            .or(inline_expr.clone())
            .recover_with(skip_then_retry_until(
                any().ignored(),
                one_of([Token::Semicolon, Token::RBrace]).ignored(),
            ))
    })
}

/// Simplified inline expression parser for simple cases
/// 
/// This is a non-recursive parser for simple expressions that don't require
/// the full complexity of the main parser. Used in contexts where we need
/// basic expression parsing without deep nesting.
pub fn inline_expr_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    // ===== SIMPLE ATOMIC EXPRESSIONS =====
    
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

    // ===== SIMPLE BINARY EXPRESSIONS =====
    // Pattern: identifier operator literal
    
    let simple_binary = ident.clone()
        .then(bin_op_parser())
        .then(lit.clone())
        .map_with(|((left, op), right), e| Expr {
            id: NodeId::new(),
            kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
            span: e.span(),
        });

    // ===== SIMPLE BLOCK HANDLING =====
    // Basic block parser that skips block contents
    
    let simple_block = any()
        .filter(|t| matches!(t, Token::LBrace))
        .ignore_then(
            any()
                .filter(|t| !matches!(t, Token::RBrace))
                .repeated()
        )
        .then_ignore(any().filter(|t| matches!(t, Token::RBrace)))
        .map_with(|_, e| crate::parser::block::Block {
            stmts: LocalVec::new(),
            id: NodeId::new(), 
            span: e.span(),
        });

    // ===== SIMPLE IF EXPRESSIONS =====
    // If expressions with simple blocks
    
    let simple_condition = simple_binary.clone().or(lit.clone()).or(ident.clone());
    
    let simple_if = just(Token::If)
        .ignore_then(simple_condition)
        .then(simple_block)
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
                else_block_opt.map(|block| Box::new(Expr {
                    id: NodeId::new(),
                    kind: ExprKind::Block(Box::new(block)),
                    span: e.span(),
                }))
            ),
            span: e.span(),
        });

    simple_if.or(simple_binary).or(lit).or(ident)
}
