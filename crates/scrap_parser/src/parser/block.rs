use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

use crate::{Span, ast::NodeId, utils::LocalVec};

use super::stmt::{Stmt, parse_stmt};

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: LocalVec<Stmt>,
    pub id: NodeId,
    pub span: Span,
}

pub fn block_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Block, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    recursive(|_| {
        // An item inside the block is one of two things:

        let any_other_token = parse_stmt()
            .repeated()
            .collect::<LocalVec<_>>()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .map_with(|stmts, e| Block {
                stmts,
                id: NodeId::new(),
                span: e.span(),
            })
            .recover_with(via_parser(nested_delimiters(
                Token::LBracket,
                Token::RBracket,
                [
                    (Token::LParen, Token::RParen),
                    (Token::LBracket, Token::RBracket),
                ],
                |span| Block {
                    stmts: LocalVec::new(),
                    id: NodeId::new(),
                    span,
                },
            )));
        // An item is either a nested block or any other token.
        any_other_token
    })
}
