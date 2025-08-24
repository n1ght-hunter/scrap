use std::fmt;

use chumsky::util::MaybeRef;

use super::pattern::Pattern;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Reason<'a, T> {
    /// An unexpected input was found
    ExpectedFound {
        /// The tokens expected
        expected: Vec<Pattern<'a, T>>,
        /// The tokens found
        found: Option<MaybeRef<'a, T>>,
    },
    /// An error with a custom message
    Custom(String),
}

impl<'a, T> Reason<'a, T> {
    /// Return the token that was found by this error reason. `None` implies that the end of input was expected.
    pub fn found(&self) -> Option<&T> {
        match self {
            Self::ExpectedFound { found, .. } => found.as_deref(),
            Self::Custom(_) => None,
        }
    }

    /// Convert this reason into an owned version of itself by cloning any borrowed internal tokens, if necessary.
    pub fn into_owned<'b>(self) -> Reason<'b, T>
    where
        T: Clone,
    {
        match self {
            Self::ExpectedFound { found, expected } => Reason::ExpectedFound {
                expected: expected.into_iter().map(Pattern::into_owned).collect(),
                found: found.map(MaybeRef::into_owned),
            },
            Self::Custom(msg) => Reason::Custom(msg),
        }
    }

    pub(super) fn take_found(&mut self) -> Option<MaybeRef<'a, T>> {
        match self {
            Reason::ExpectedFound { found, .. } => found.take(),
            Reason::Custom(_) => None,
        }
    }

    /// Transform this `RichReason`'s tokens using the given function.
    ///
    /// This is useful when you wish to combine errors from multiple compilation passes (lexing and parsing, say) where
    /// the token type for each pass is different (`char` vs `MyToken`, say).
    pub fn map_token<U, F: FnMut(T) -> U>(self, mut f: F) -> Reason<'a, U>
    where
        T: Clone,
    {
        match self {
            Reason::ExpectedFound { expected, found } => Reason::ExpectedFound {
                expected: expected
                    .into_iter()
                    .map(|pat| pat.map_token(&mut f))
                    .collect(),
                found: found.map(|found| f(found.into_inner()).into()),
            },
            Reason::Custom(msg) => Reason::Custom(msg),
        }
    }

    pub(super) fn inner_fmt<S>(
        &self,
        f: &mut fmt::Formatter<'_>,
        mut fmt_token: impl FnMut(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
        mut fmt_span: impl FnMut(&S, &mut fmt::Formatter<'_>) -> fmt::Result,
        span: Option<&S>,
        context: &[(Pattern<'a, T>, S)],
    ) -> fmt::Result {
        match self {
            Reason::ExpectedFound { expected, found } => {
                write!(f, "found ")?;
                write_token(f, &mut fmt_token, found.as_deref())?;
                if let Some(span) = span {
                    write!(f, " at ")?;
                    fmt_span(span, f)?;
                }
                write!(f, " expected ")?;
                match &expected[..] {
                    [] => write!(f, "something else")?,
                    [expected] => expected.write(f, &mut fmt_token)?,
                    _ => {
                        for expected in &expected[..expected.len() - 1] {
                            expected.write(f, &mut fmt_token)?;
                            write!(f, ", ")?;
                        }
                        write!(f, "or ")?;
                        expected.last().unwrap().write(f, &mut fmt_token)?;
                    }
                }
            }
            Reason::Custom(msg) => {
                write!(f, "{msg}")?;
                if let Some(span) = span {
                    write!(f, " at ")?;
                    fmt_span(span, f)?;
                }
            }
        }
        for (l, s) in context {
            write!(f, " in ")?;
            l.write(f, &mut fmt_token)?;
            write!(f, " at ")?;
            fmt_span(s, f)?;
        }
        Ok(())
    }
}

impl<T> Reason<'_, T>
where
    T: PartialEq,
{
    #[inline]
    pub(super) fn flat_merge(self, other: Self) -> Self {
        match (self, other) {
            // Prefer first error, if ambiguous
            (a @ Reason::Custom(_), _) => a,
            (_, b @ Reason::Custom(_)) => b,
            (
                Reason::ExpectedFound {
                    expected: mut this_expected,
                    found,
                },
                Reason::ExpectedFound {
                    expected: mut other_expected,
                    ..
                },
            ) => {
                // Try to avoid allocations if we possibly can by using the longer vector
                if other_expected.len() > this_expected.len() {
                    core::mem::swap(&mut this_expected, &mut other_expected);
                }
                for expected in other_expected {
                    if !this_expected[..].contains(&expected) {
                        this_expected.push(expected);
                    }
                }
                Reason::ExpectedFound {
                    expected: this_expected,
                    found,
                }
            }
        }
    }
}

impl<T> fmt::Display for Reason<'_, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner_fmt(f, T::fmt, |_: &(), _| Ok(()), None, &[])
    }
}


fn write_token<T>(
    f: &mut fmt::Formatter,
    mut fmt_token: impl FnMut(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
    tok: Option<&T>,
) -> fmt::Result {
    match tok {
        Some(tok) => {
            write!(f, "'")?;
            fmt_token(tok, f)?;
            write!(f, "'")
        }
        None => write!(f, "end of input"),
    }
}