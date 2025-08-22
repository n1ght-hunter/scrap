use crate::{Span, Spanned};

use chumsky::{input::ValueInput, prelude::*};
use scrap_lexer::Token;

pub type BinOp = Spanned<BinOpKind>;

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

pub fn bin_op_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, BinOp, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
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
