pub mod kind;
pub mod pattern;
pub mod reason;

use std::{fmt, path::Path};

use ariadne::Label;
use chumsky::{input::Input, span::Span, util::MaybeRef};
use kind::ReportKind;
use pattern::Pattern;
use reason::Reason;

pub struct ParseError<'a, T, S = crate::Span> {
    span: S,
    kind: ReportKind,
    help: Option<String>,
    reason: Box<Reason<'a, T>>,
    context: Vec<(Pattern<'a, T>, S)>,
}

impl<T, S> ParseError<'_, T, S> {
    fn inner_fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
        fmt_token: impl FnMut(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
        fmt_span: impl FnMut(&S, &mut fmt::Formatter<'_>) -> fmt::Result,
        with_spans: bool,
    ) -> fmt::Result {
        self.reason.inner_fmt(
            f,
            fmt_token,
            fmt_span,
            if with_spans { Some(&self.span) } else { None },
            &self.context,
        )
    }
}

impl<'a, T, S> ParseError<'a, T, S> {
    /// Set the help message for this error.
    fn with_help(mut self, msg: impl Into<String>) -> Self {
        self.help = Some(msg.into());
        self
    }

    /// Set the kind of this error.
    pub fn with_kind(mut self, kind: ReportKind) -> Self {
        self.kind = kind;
        self
    }
}

impl<'a, T, S> ParseError<'a, T, S> {
    /// Create an error with a custom message and span
    #[inline]
    pub fn custom<M: ToString>(span: S, msg: M) -> Self {
        ParseError {
            span,
            reason: Box::new(Reason::Custom(msg.to_string())),
            context: Vec::new(),
            kind: ReportKind::Error,
            help: None,
        }
    }

    #[inline]
    pub fn custom_with_kind<M: ToString>(span: S, msg: M, kind: ReportKind) -> Self {
        ParseError {
            span,
            reason: Box::new(Reason::Custom(msg.to_string())),
            context: Vec::new(),
            kind,
            help: None,
        }
    }

    /// Get the span associated with this error.
    ///
    /// If the span type is unspecified, it is [`SimpleSpan`].
    pub fn span(&self) -> &S {
        &self.span
    }

    /// Get the reason for this error.
    pub fn reason(&self) -> &Reason<'a, T> {
        &self.reason
    }

    /// Take the reason from this error.
    pub fn into_reason(self) -> Reason<'a, T> {
        *self.reason
    }

    /// Get the token found by this error when parsing. `None` implies that the error expected the end of input.
    pub fn found(&self) -> Option<&T> {
        self.reason.found()
    }

    /// Get the help message associated with this error.
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    /// Return an iterator over the labelled contexts of this error, from least general to most.
    ///
    /// 'Context' here means parser patterns that the parser was in the process of parsing when the error occurred. To
    /// add labelled contexts, see [`Parser::labelled`].
    pub fn contexts(&self) -> impl Iterator<Item = (&Pattern<'a, T>, &S)> {
        self.context.iter().map(|(l, s)| (l, s))
    }

    /// Convert this error into an owned version of itself by cloning any borrowed internal tokens, if necessary.
    pub fn into_owned<'b>(self) -> ParseError<'b, T, S>
    where
        T: Clone,
    {
        ParseError {
            reason: Box::new(self.reason.into_owned()),
            context: self
                .context
                .into_iter()
                .map(|(p, s)| (p.into_owned(), s))
                .collect(),
            ..self
        }
    }

    /// Get an iterator over the expected items associated with this error
    pub fn expected(&self) -> impl ExactSizeIterator<Item = &Pattern<'a, T>> {
        match &*self.reason {
            Reason::ExpectedFound { expected, .. } => expected.iter(),
            Reason::Custom(_) => [].iter(),
        }
    }

    /// Transform this error's tokens using the given function.
    ///
    /// This is useful when you wish to combine errors from multiple compilation passes (lexing and parsing, say) where
    /// the token type for each pass is different (`char` vs `MyToken`, say).
    pub fn map_token<U, F: FnMut(T) -> U>(self, mut f: F) -> ParseError<'a, U, S>
    where
        T: Clone,
    {
        ParseError {
            reason: Box::new(self.reason.map_token(&mut f)),
            context: self
                .context
                .into_iter()
                .map(|(p, s)| (p.map_token(&mut f), s))
                .collect(),
            span: self.span,
            kind: self.kind,
            help: self.help,
        }
    }
}

impl<'a, T, S: Span<Offset = usize>> ParseError<'a, T, S>
where
    T: fmt::Display,
    S: fmt::Display,
{
    pub fn print(
        &self,
        filename: impl AsRef<Path>,
        source: &ariadne::Source<&String>,
    ) -> Result<(), std::io::Error> {
        let filename = &filename.as_ref().to_string_lossy().to_string();
        let span = Span::start(self.span())..=Span::end(self.span());
        let mut error = ariadne::Report::build(self.kind.into(), (filename, span.clone()))
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_message(self.to_string())
            .with_label(
                Label::new((filename, span))
                    .with_message(self.reason().to_string())
                    .with_color(self.kind.color()),
            )
            .with_labels(self.contexts().map(|(label, span)| {
                Label::new((filename, (Span::start(span)..=Span::end(span))))
                    .with_message(format!("while parsing this {label}"))
                    .with_color(self.kind.color())
            }));

        if let Some(help) = self.help() {
            error = error.with_help(help);
        }

        error.finish().print((filename, source))
    }
}

impl<'a, I: Input<'a>> chumsky::error::Error<'a, I> for ParseError<'a, I::Token, I::Span>
where
    I::Token: PartialEq,
{
    #[inline]
    fn merge(self, other: Self) -> Self {
        let new_reason = self.reason.flat_merge(*other.reason);
        Self {
            span: self.span,
            reason: Box::new(new_reason),
            context: self.context,
            kind: self.kind,
            help: self.help,
        }
    }
}

impl<'a, I: Input<'a>, L> chumsky::error::LabelError<'a, I, L> for ParseError<'a, I::Token, I::Span>
where
    I::Token: PartialEq,
    L: Into<Pattern<'a, I::Token>>,
{
    #[inline]
    fn expected_found<E: IntoIterator<Item = L>>(
        expected: E,
        found: Option<MaybeRef<'a, I::Token>>,
        span: I::Span,
    ) -> Self {
        Self {
            span,
            reason: Box::new(Reason::ExpectedFound {
                expected: expected.into_iter().map(|tok| tok.into()).collect(),
                found,
            }),
            context: Vec::new(),
            kind: ReportKind::Error,
            help: None,
        }
    }

    #[inline]
    fn merge_expected_found<E: IntoIterator<Item = L>>(
        mut self,
        new_expected: E,
        new_found: Option<MaybeRef<'a, I::Token>>,
        _span: I::Span,
    ) -> Self {
        match &mut *self.reason {
            Reason::ExpectedFound { expected, found } => {
                for new_expected in new_expected {
                    let new_expected = new_expected.into();
                    if !expected[..].contains(&new_expected) {
                        expected.push(new_expected);
                    }
                }
                *found = found.take().or(new_found); //land
            }
            Reason::Custom(_) => {}
        }
        // TOOD: Merge contexts
        self
    }

    #[inline]
    fn replace_expected_found<E: IntoIterator<Item = L>>(
        mut self,
        new_expected: E,
        new_found: Option<MaybeRef<'a, I::Token>>,
        span: I::Span,
    ) -> Self {
        self.span = span;
        match &mut *self.reason {
            Reason::ExpectedFound { expected, found } => {
                expected.clear();
                expected.extend(new_expected.into_iter().map(|tok| tok.into()));
                *found = new_found;
            }
            _ => {
                self.reason = Box::new(Reason::ExpectedFound {
                    expected: new_expected.into_iter().map(|tok| tok.into()).collect(),
                    found: new_found,
                });
            }
        }
        self.context.clear();
        self
    }

    #[inline]
    fn label_with(&mut self, label: L) {
        // Opportunistically attempt to reuse allocations if we can
        match &mut *self.reason {
            Reason::ExpectedFound { expected, found: _ } => {
                expected.clear();
                expected.push(label.into());
            }
            Reason::Custom(_) => {
                self.reason = Box::new(Reason::ExpectedFound {
                    expected: vec![label.into()],
                    found: self.reason.take_found(),
                });
            }
        }
    }

    #[inline]
    fn in_context(&mut self, label: L, span: I::Span) {
        let label = label.into();
        if self.context.iter().all(|(l, _)| l != &label) {
            self.context.push((label, span));
        }
    }
}

impl<T, S> fmt::Debug for ParseError<'_, T, S>
where
    T: fmt::Debug,
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner_fmt(f, T::fmt, S::fmt, true)
    }
}

impl<T, S> fmt::Display for ParseError<'_, T, S>
where
    T: fmt::Display,
    S: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner_fmt(f, T::fmt, S::fmt, false)
    }
}
