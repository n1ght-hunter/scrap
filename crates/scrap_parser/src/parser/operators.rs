use crate::Spanned;

use chumsky::prelude::*;
use scrap_lexer::Token;

use super::{ScrapInput, ScrapParser};

/// A binary operator with its source location span.
/// This matches the Rust AST pattern of wrapping operator kinds with span information.
pub type BinOp = Spanned<BinOpKind>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AssocOp {
    /// A binary op.
    Binary(BinOpKind),
    /// `?=` where ? is one of the assignable BinOps
    AssignOp(AssignOpKind),
    /// `=`
    Assign,
}

/// Assignment operator kinds, following Rust AST enum structure exactly.
/// These represent the different types of assignment operations available in the language.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AssignOpKind {
    /// The `+=` operator (addition)
    AddAssign,
    /// The `-=` operator (subtraction)
    SubAssign,
    /// The `*=` operator (multiplication)
    MulAssign,
    /// The `/=` operator (division)
    DivAssign,
    /// The `%=` operator (modulus)
    RemAssign,
    /// The `^=` operator (bitwise xor)
    BitXorAssign,
    /// The `&=` operator (bitwise and)
    BitAndAssign,
    /// The `|=` operator (bitwise or)
    BitOrAssign,
    /// The `<<=` operator (shift left)
    ShlAssign,
    /// The `>>=` operator (shift right)
    ShrAssign,
}

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
pub fn bin_op_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, BinOpKind>
where
    I: ScrapInput<'tokens, 'src>,
{
    choice((
        just(Token::Add).to(BinOpKind::Add),
        just(Token::Sub).to(BinOpKind::Sub),
        just(Token::Mul).to(BinOpKind::Mul),
        just(Token::Div).to(BinOpKind::Div),
        just(Token::Rem).to(BinOpKind::Rem),
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
    ))
}

pub fn assign_op_parser<'tokens, 'src: 'tokens, I>()
-> impl ScrapParser<'tokens, 'src, I, AssignOpKind>
where
    I: ScrapInput<'tokens, 'src>,
{
    choice((
        just(Token::AddAssign).to(AssignOpKind::AddAssign),
        just(Token::SubAssign).to(AssignOpKind::SubAssign),
        just(Token::MulAssign).to(AssignOpKind::MulAssign),
        just(Token::DivAssign).to(AssignOpKind::DivAssign),
        just(Token::RemAssign).to(AssignOpKind::RemAssign),
        just(Token::BitXorAssign).to(AssignOpKind::BitXorAssign),
        just(Token::BitAndAssign).to(AssignOpKind::BitAndAssign),
        just(Token::BitOrAssign).to(AssignOpKind::BitOrAssign),
        just(Token::ShlAssign).to(AssignOpKind::ShlAssign),
        just(Token::ShrAssign).to(AssignOpKind::ShrAssign),
    ))
}

fn assoc_op_parser<'tokens, 'src: 'tokens, I>() -> impl ScrapParser<'tokens, 'src, I, AssocOp>
where
    I: ScrapInput<'tokens, 'src>,
{
    choice((
        just(Token::Assign).to(AssocOp::Assign),
        bin_op_parser().map(AssocOp::Binary),
        assign_op_parser().map(AssocOp::AssignOp),
    ))
}

impl AssocOp {
    pub fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Assign => Some(AssocOp::Assign),
            Token::Add => Some(AssocOp::Binary(BinOpKind::Add)),
            Token::Sub => Some(AssocOp::Binary(BinOpKind::Sub)),
            Token::Mul => Some(AssocOp::Binary(BinOpKind::Mul)),
            Token::Div => Some(AssocOp::Binary(BinOpKind::Div)),
            Token::Rem => Some(AssocOp::Binary(BinOpKind::Rem)),
            Token::And => Some(AssocOp::Binary(BinOpKind::And)),
            Token::Or => Some(AssocOp::Binary(BinOpKind::Or)),
            Token::BitXor => Some(AssocOp::Binary(BinOpKind::BitXor)),
            Token::BitAnd => Some(AssocOp::Binary(BinOpKind::BitAnd)),
            Token::BitOr => Some(AssocOp::Binary(BinOpKind::BitOr)),
            Token::Shl => Some(AssocOp::Binary(BinOpKind::Shl)),
            Token::Shr => Some(AssocOp::Binary(BinOpKind::Shr)),
            Token::Eq => Some(AssocOp::Binary(BinOpKind::Eq)),
            Token::Lt => Some(AssocOp::Binary(BinOpKind::Lt)),
            Token::Le => Some(AssocOp::Binary(BinOpKind::Le)),
            Token::Ne => Some(AssocOp::Binary(BinOpKind::Ne)),
            Token::Ge => Some(AssocOp::Binary(BinOpKind::Ge)),
            Token::Gt => Some(AssocOp::Binary(BinOpKind::Gt)),
            Token::AddAssign => Some(AssocOp::AssignOp(AssignOpKind::AddAssign)),
            Token::SubAssign => Some(AssocOp::AssignOp(AssignOpKind::SubAssign)),
            Token::MulAssign => Some(AssocOp::AssignOp(AssignOpKind::MulAssign)),
            Token::DivAssign => Some(AssocOp::AssignOp(AssignOpKind::DivAssign)),
            Token::RemAssign => Some(AssocOp::AssignOp(AssignOpKind::RemAssign)),
            Token::BitXorAssign => Some(AssocOp::AssignOp(AssignOpKind::BitXorAssign)),
            Token::BitAndAssign => Some(AssocOp::AssignOp(AssignOpKind::BitAndAssign)),
            Token::BitOrAssign => Some(AssocOp::AssignOp(AssignOpKind::BitOrAssign)),
            Token::ShlAssign => Some(AssocOp::AssignOp(AssignOpKind::ShlAssign)),
            Token::ShrAssign => Some(AssocOp::AssignOp(AssignOpKind::ShrAssign)),
            _ => None,
        }
    }
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
    let mul_div_ops = just(Token::Mul).or(just(Token::Div));

    base_parser.clone().foldl_with(
        mul_div_ops.then(base_parser).repeated(),
        |lhs, (op_token, rhs), e| {
            let op = Spanned {
                node: match op_token {
                    Token::Mul => BinOpKind::Mul,
                    Token::Div => BinOpKind::Div,
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
    let add_sub_ops = just(Token::Add).or(just(Token::Sub));

    base_parser.clone().foldl_with(
        add_sub_ops.then(base_parser).repeated(),
        |lhs, (op_token, rhs), e| {
            let op = Spanned {
                node: match op_token {
                    Token::Add => BinOpKind::Add,
                    Token::Sub => BinOpKind::Sub,
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

pub fn ops_parser<'tokens, 'src: 'tokens, I>(
    base_parser: impl ScrapParser<'tokens, 'src, I, Expr>,
) -> impl ScrapParser<'tokens, 'src, I, Expr>
where
    I: ScrapInput<'tokens, 'src>,
{
    // Product ops (multiply and divide) have equal precedence

    let product = product_parser(base_parser);

    // Sum ops (add and subtract) have equal precedence
    
    let sum = sum_parser(product);
    // Comparison ops (equal, not-equal) have equal precedence

    let compare = comparison_parser(sum);

    compare.labelled("expression").as_context()
}
