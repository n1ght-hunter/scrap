pub use logos::Logos;
use scrap_diagnostics::{AnnotationKind, Level, Snippet};
use scrap_macros::expand_tokens;
use scrap_span::Spanned;

use crate::error::LexingError;

pub mod error;
pub mod token_stream;

expand_tokens! {

#[derive(Debug, PartialEq, Clone)]
pub enum KeyWord {
   #[token("enum")]
    Enum,
    #[token("struct")]
    Struct,
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("return")]
    Return,
    #[token("mod")]
    Mod,
    #[token("use")]
    Use,
    #[token("extern")]
    Extern,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    #[regex(r#""(\\.|[^"\\])*""#)]
    Str,

    #[regex(r"[0-9]+\.[0-9]+")]
    Float,

    #[regex(r"[0-9]+")]
    Int,

    #[token("false")]
    #[token("true")]
    Bool,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("%")]
    Rem,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("^")]
    BitXor,
    #[token("&")]
    BitAnd,
    #[token("|")]
    BitOr,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token("==")]
    Eq,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token("!=")]
    Ne,
    #[token(">=")]
    Ge,
    #[token(">")]
    Gt,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignOp {
    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("*=")]
    MulAssign,
    #[token("/=")]
    DivAssign,
    #[token("%=")]
    RemAssign,
    #[token("&&=")]
    AndAssign,
    #[token("||=")]
    OrAssign,
    #[token("^=")]
    BitXorAssign,
    #[token("&=")]
    BitAndAssign,
    #[token("|=")]
    BitOrAssign,
    #[token("<<=")]
    ShlAssign,
    #[token(">>=")]
    ShrAssign,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    #[token("->")]
    Arrow,
    #[token("=")]
    Assign,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Delimiter {
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token(";")]
    Semicolon,
    #[token("!")]
    Bang,
    #[token(".")]
    Dot,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Visibility {
    #[token("pub")]
    Pub,
}


#[derive(Debug, PartialEq, Clone)]
pub enum Trivia {
      // Preserve whitespace (for Rowan parser)
    #[regex(r"[ \t\r\n\f]+")]
    #[display("<whitespace>")]
    Whitespace,

    // Preserve comments (for Rowan parser)
    #[regex(r"///[^\r\n]*", allow_greedy = true)]
    #[display("<doc_comment>")]
    DocComment,
    #[regex(r"//[^\r\n]*", allow_greedy = true)]
    #[display("<comment>")]
    Comment,
}

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy, Hash, salsa::Update, serde::Serialize, serde::Deserialize)]
#[logos(error(LexingError, LexingError::from_lexer))]
pub enum Token {
    Dummy,
    Eof,
}

}

impl Token {
    #[inline(always)]
    pub const fn dummy() -> Self {
        Token::Dummy
    }
}

#[salsa::tracked(debug, persist)]
pub struct LexedTokens<'db> {
    pub tokens: token_stream::TokenStream<'db>,
}

#[salsa::tracked(persist)]
pub fn lex_file<'db>(
    db: &'db dyn scrap_shared::Db,
    file: scrap_shared::salsa::InputFile<'db>,
) -> LexedTokens<'db> {
    let content = file.content(db);
    let token_iter = Token::lexer(content)
        .spanned()
        .filter_map(|(new_tok, new_span)| {
            let span = scrap_span::Span::new(db, new_span.start, new_span.end);
            match new_tok {
                Ok(new_tok) => Some(Spanned::new(new_tok, span)),
                Err(e) => {
                    db.dcx().emit_err(
                        Level::ERROR.primary_title(e.to_string()).element(
                            Snippet::source(content)
                                .path(file.path(db).to_string_lossy())
                                .annotation(
                                    AnnotationKind::Primary
                                        .span(span.to_range(db))
                                        .label(e.to_string()),
                                ),
                        ),
                    );
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    LexedTokens::new(db, token_stream::TokenStream::new(token_iter))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let file = std::fs::read_to_string("../../tests/basic.sc").unwrap();

        let mut lexer = Token::lexer(&file);

        let mut has_err = false;

        let mut tokens = Vec::new();

        while let Some(res_token) = lexer.next() {
            match res_token {
                Ok(token) => {
                    tokens.push(token);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    has_err = true;
                }
            }
        }

        assert!(!has_err);

        insta::assert_debug_snapshot!(tokens);
    }
}
