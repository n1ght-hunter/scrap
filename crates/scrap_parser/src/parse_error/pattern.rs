use std::{borrow::Cow, fmt};

use chumsky::{
    DefaultExpected,
    input::StrInput,
    text::{self, Char},
    util::MaybeRef,
};
use tracing::warn;

/// An expected pattern for a [`Rich`] error.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Pattern<'a, T> {
    /// A specific token.
    Token(MaybeRef<'a, T>),
    /// A labelled pattern.
    Label(Cow<'a, str>),
    /// A specific keyword.
    Identifier(String),
    /// Anything other than the end of input.
    Any,
    /// Something other than the provided input.
    SomethingElse,
    /// The end of input.
    EndOfInput,
}

impl<'a, T> From<DefaultExpected<'a, T>> for Pattern<'a, T> {
    fn from(expected: DefaultExpected<'a, T>) -> Self {
        match expected {
            DefaultExpected::Token(tok) => Self::Token(tok),
            DefaultExpected::Any => Self::Any,
            DefaultExpected::SomethingElse => Self::SomethingElse,
            DefaultExpected::EndOfInput => Self::EndOfInput,
            _ => {
                warn!(target: "scrap_parser", "Unexpected pattern in DefaultExpected");
                Self::SomethingElse
            }
        }
    }
}

impl<'a, I: StrInput<'a>, T> From<text::TextExpected<'a, I>> for Pattern<'a, T>
where
    I::Token: Char,
{
    fn from(expected: text::TextExpected<'a, I>) -> Self {
        match expected {
            text::TextExpected::Whitespace => Self::Label(Cow::Borrowed("whitespace")),
            text::TextExpected::InlineWhitespace => Self::Label(Cow::Borrowed("inline whitespace")),
            text::TextExpected::Newline => Self::Label(Cow::Borrowed("newline")),
            text::TextExpected::Digit(r) if r.start > 0 => {
                Self::Label(Cow::Borrowed("non-zero digit"))
            }
            text::TextExpected::Digit(_) => Self::Label(Cow::Borrowed("digit")),
            text::TextExpected::IdentifierPart => Self::Label(Cow::Borrowed("identifier")),
            text::TextExpected::Identifier(i) => Self::Identifier(I::stringify(i)),
            _ => {
                warn!(target: "scrap_parser", "Unexpected pattern in TextExpected");
                Self::SomethingElse
            }
        }
    }
}

impl<'a, T> From<MaybeRef<'a, T>> for Pattern<'a, T> {
    fn from(tok: MaybeRef<'a, T>) -> Self {
        Self::Token(tok)
    }
}

impl<T> From<&'static str> for Pattern<'_, T> {
    fn from(label: &'static str) -> Self {
        Self::Label(Cow::Borrowed(label))
    }
}

impl<T> From<String> for Pattern<'_, T> {
    fn from(label: String) -> Self {
        Self::Label(Cow::Owned(label))
    }
}

impl From<char> for Pattern<'_, char> {
    fn from(c: char) -> Self {
        Self::Token(MaybeRef::Val(c))
    }
}

impl<'a, T> Pattern<'a, T> {
    /// Transform this pattern's tokens using the given function.
    ///
    /// This is useful when you wish to combine errors from multiple compilation passes (lexing and parsing, say) where
    /// the token type for each pass is different (`char` vs `MyToken`, say).
    pub fn map_token<U, F: FnMut(T) -> U>(self, mut f: F) -> Pattern<'a, U>
    where
        T: Clone,
    {
        match self {
            Self::Token(t) => Pattern::Token(f(t.into_inner()).into()),
            Self::Label(l) => Pattern::Label(l),
            Self::Identifier(i) => Pattern::Identifier(i),
            Self::Any => Pattern::Any,
            Self::SomethingElse => Pattern::SomethingElse,
            Self::EndOfInput => Pattern::EndOfInput,
        }
    }

    /// Convert this pattern into an owned version of itself by cloning any borrowed internal tokens, if necessary.
    pub fn into_owned<'b>(self) -> Pattern<'b, T>
    where
        T: Clone,
    {
        match self {
            Self::Token(tok) => Pattern::Token(tok.into_owned()),
            Self::Label(l) => Pattern::Label(Cow::Owned(l.into_owned())),
            Self::Identifier(i) => Pattern::Identifier(i),
            Self::Any => Pattern::Any,
            Self::SomethingElse => Pattern::SomethingElse,
            Self::EndOfInput => Pattern::EndOfInput,
        }
    }

    pub(super) fn write(
        &self,
        f: &mut fmt::Formatter,
        mut fmt_token: impl FnMut(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
    ) -> fmt::Result {
        match self {
            Self::Token(tok) => {
                write!(f, "'")?;
                fmt_token(tok, f)?;
                write!(f, "'")
            }
            Self::Label(l) => write!(f, "{l}"),
            Self::Identifier(i) => write!(f, "'{i}'"),
            Self::Any => write!(f, "any"),
            Self::SomethingElse => write!(f, "something else"),
            Self::EndOfInput => write!(f, "end of input"),
        }
    }
}

impl<T> fmt::Debug for Pattern<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f, |t, f| write!(f, "{t:?}"))
    }
}

impl<T> fmt::Display for Pattern<'_, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f, |t, f| write!(f, "{t}"))
    }
}
