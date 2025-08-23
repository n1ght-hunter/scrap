use chumsky::prelude::*;
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, utils::LocalVec};

use super::{
    ScrapInput, ScrapParser,
    stmt::{Stmt, parse_stmt},
};

/// A block expression. Following Rust AST structure.
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: LocalVec<Stmt>,
    pub id: NodeId,
    pub span: Span,
}

pub fn block_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, Block>
where
    I: ScrapInput<'tokens, 'src>,
{
    recursive(|_block| {
        // Parse statements with better structure:
        // - All statements except the last must have semicolons
        // - The last statement can be an expression without semicolon (becomes block value)

        let statements_with_semicolons = parse_stmt()
            .repeated()
            .collect::<LocalVec<_>>()
            .labelled("block contents");

        statements_with_semicolons
            .delimited_by(
                just(Token::LBrace).labelled("opening brace"),
                just(Token::RBrace).labelled("closing brace"),
            )
            .map_with(|stmts, e| Block {
                stmts,
                id: e.state().new_node_id(),
                span: e.span(),
            })
            .labelled("block")
            .as_context()
    })
}
