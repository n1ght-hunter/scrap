//! Defines the structures and rendering logic for compiler diagnostics.
//! This is the "presentation layer" for errors.

use std::{
    borrow::Cow,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub use annotate_snippets::{self, Annotation, AnnotationKind, Level, Snippet};
pub use anstream;
pub use anstyle;

use annotate_snippets::{Group, Renderer, renderer::DecorStyle};
use scrap_errors::ErrorGuaranteed;

/// Owned, lifetime-free emitter. The whole emitter is a single
/// `Arc<DiagnosticInner>`, so cloning (e.g. per rayon worker via
/// [`Db::fork`](scrap_shared)) is a refcount bump — the `Renderer` and config
/// flags live behind the `Arc` rather than being copied per clone. This also
/// lets `DiagnosticEmitter` be embedded in a long-lived `Db` without lifetime
/// gymnastics on the borrow returned by `dcx()`.
#[derive(Clone)]
pub struct DiagnosticEmitter {
    inner: Arc<DiagnosticInner>,
    /// Whether diagnostics should be printed to stderr at emit time.
    auto_render: bool,
    /// Whether all unrendered diagnostics should be flushed when the last
    /// emitter handle is dropped.
    render_on_drop: bool,
}

impl Default for DiagnosticEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DiagnosticEmitter {
    fn drop(&mut self) {
        if self.render_on_drop && Arc::strong_count(&self.inner) == 1 {
            self.render_all();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Emitted {
    Yes,
    No,
}

/// A stored diagnostic, either already rendered to text or kept as an owned
/// group to be rendered lazily at display time.
enum StoredDiag {
    /// Pre-rendered text. A `&'static str` input stays `Cow::Borrowed` and
    /// never allocates.
    Rendered {
        emitted: Emitted,
        text: Cow<'static, str>,
    },
    /// Owned group; rendered on demand at `render_all` / drop time so emitting
    /// a diagnostic that is later cleared costs no render.
    Lazy {
        emitted: Emitted,
        group: Group<'static>,
    },
}

impl StoredDiag {
    fn emitted(&self) -> Emitted {
        match self {
            Self::Rendered { emitted, .. } | Self::Lazy { emitted, .. } => *emitted,
        }
    }

    fn set_emitted(&mut self, value: Emitted) {
        match self {
            Self::Rendered { emitted, .. } | Self::Lazy { emitted, .. } => *emitted = value,
        }
    }

    /// Rendered text, read-only. A `Rendered` entry borrows its stored text
    /// (no alloc); a `Lazy` entry renders its group on the fly into an owned
    /// string. The caller marks the entry emitted afterwards, so a given entry
    /// is never rendered twice.
    fn render<'r>(&'r self, renderer: &Renderer) -> Cow<'r, str> {
        match self {
            Self::Rendered { text, .. } => Cow::Borrowed(text),
            Self::Lazy { group, .. } => Cow::Owned(renderer.render(std::slice::from_ref(group))),
        }
    }
}

struct DiagnosticInner {
    errors: parking_lot::Mutex<Vec<StoredDiag>>,
    warnings: parking_lot::Mutex<Vec<StoredDiag>>,
    others: parking_lot::Mutex<Vec<StoredDiag>>,
    renderer: Renderer,
}

impl DiagnosticInner {
    fn push(&self, level: Level<'_>, diag: StoredDiag) {
        match level {
            Level::ERROR => self.errors.lock().push(diag),
            Level::WARNING => self.warnings.lock().push(diag),
            _ => self.others.lock().push(diag),
        }
    }

    fn has_errors(&self) -> bool {
        !self.errors.lock().is_empty()
    }

    fn has_warnings(&self) -> bool {
        !self.warnings.lock().is_empty()
    }

    fn has_unrendered(&self) -> bool {
        let check = |input: &parking_lot::Mutex<Vec<StoredDiag>>| {
            let guard = input.lock();
            guard.iter().any(|d| d.emitted() == Emitted::No)
        };
        let result = AtomicBool::new(false);
        rayon::scope(|s| {
            s.spawn(|_| {
                if check(&self.errors) {
                    result.store(true, Ordering::Relaxed);
                }
            });
            s.spawn(|_| {
                if check(&self.warnings) {
                    result.store(true, Ordering::Relaxed);
                }
            });
            s.spawn(|_| {
                if check(&self.others) {
                    result.store(true, Ordering::Relaxed);
                }
            });
        });
        result.load(Ordering::Relaxed)
    }

    fn clear(&self) {
        self.errors.lock().clear();
        self.warnings.lock().clear();
        self.others.lock().clear();
    }

    fn counts(&self) -> (usize, usize, usize) {
        (
            self.errors.lock().len(),
            self.warnings.lock().len(),
            self.others.lock().len(),
        )
    }

    fn all_non_rendered(
        &self,
        print: impl Fn(Level, &mut dyn Iterator<Item = Cow<'_, str>>) + Sync + Send,
    ) {
        // Sequential within a level: each level already runs on its own
        // `rayon::scope` thread. Both phases run under one held lock, so `print`
        // receives a borrowing iterator (no intermediate `Vec`).
        let collect = |level: Level, input: &parking_lot::Mutex<Vec<StoredDiag>>| {
            let mut guard = input.lock();
            // 1. Render unemitted diagnostics on the fly and hand them to
            //    `print` as an iterator.
            let mut texts = guard
                .iter()
                .filter(|d| d.emitted() == Emitted::No)
                .map(|d| d.render(&self.renderer));
            print(level, &mut texts);
            // 2. Mark them emitted (the filter above prevents double-render).
            for d in guard.iter_mut() {
                if d.emitted() == Emitted::No {
                    d.set_emitted(Emitted::Yes);
                }
            }
        };
        rayon::scope(|s| {
            s.spawn(|_| collect(Level::ERROR, &self.errors));
            s.spawn(|_| collect(Level::WARNING, &self.warnings));
            // info is used for "other" diagnostics
            s.spawn(|_| collect(Level::INFO, &self.others));
        });
    }
}

impl DiagnosticEmitter {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DiagnosticInner {
                errors: parking_lot::Mutex::default(),
                warnings: parking_lot::Mutex::default(),
                others: parking_lot::Mutex::default(),
                renderer: Renderer::styled().decor_style(DecorStyle::Unicode),
            }),
            auto_render: false,
            render_on_drop: true,
        }
    }

    pub fn with_auto_render(mut self, auto_render: bool) -> Self {
        self.auto_render = auto_render;
        self
    }

    pub fn with_render_on_drop(mut self, render_on_drop: bool) -> Self {
        self.render_on_drop = render_on_drop;
        self
    }

    pub fn has_errors(&self) -> bool {
        self.inner.has_errors()
    }

    pub fn has_warnings(&self) -> bool {
        self.inner.has_warnings()
    }

    pub fn has_unrendered(&self) -> bool {
        self.inner.has_unrendered()
    }

    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Returns the counts of (errors, warnings, others)
    pub fn counts(&self) -> (usize, usize, usize) {
        self.inner.counts()
    }

    pub fn emit_err(&self, diag: Group<'_>) -> ErrorGuaranteed {
        self.emit(Level::ERROR, diag);
        #[allow(deprecated)]
        ErrorGuaranteed::unchecked_error_guaranteed()
    }

    /// Render `diag` immediately and store the resulting text.
    ///
    /// Use this for groups built from borrowed/`format!`-ed data, which cannot
    /// outlive the call. For owned `Group<'static>` prefer [`Self::emit_lazy`].
    pub fn emit(&self, level: Level<'_>, diag: Group<'_>) {
        let text = self.inner.renderer.render(std::slice::from_ref(&diag));
        let emitted = if self.auto_render {
            anstream::eprintln!("{text}");
            Emitted::Yes
        } else {
            Emitted::No
        };
        self.inner.push(
            level,
            StoredDiag::Rendered {
                emitted,
                text: Cow::Owned(text),
            },
        );
    }

    /// Store an owned group and defer rendering until display time, so a
    /// diagnostic that is later cleared costs no render.
    pub fn emit_lazy(&self, level: Level<'_>, diag: Group<'static>) {
        if self.auto_render {
            let text = self.inner.renderer.render(std::slice::from_ref(&diag));
            anstream::eprintln!("{text}");
            self.inner.push(
                level,
                StoredDiag::Rendered {
                    emitted: Emitted::Yes,
                    text: Cow::Owned(text),
                },
            );
        } else {
            self.inner.push(
                level,
                StoredDiag::Lazy {
                    emitted: Emitted::No,
                    group: diag,
                },
            );
        }
    }

    /// Store an already-rendered message. A `&'static str` stays borrowed and
    /// never allocates or invokes the renderer.
    pub fn emit_rendered(&self, level: Level<'_>, msg: impl Into<Cow<'static, str>>) {
        let text = msg.into();
        let emitted = if self.auto_render {
            anstream::eprintln!("{text}");
            Emitted::Yes
        } else {
            Emitted::No
        };
        self.inner
            .push(level, StoredDiag::Rendered { emitted, text });
    }

    pub fn render_all(&self) {
        self.inner.all_non_rendered(|_level, texts| {
            for text in texts {
                anstream::eprintln!("{text}");
            }
        });
    }

    /// Renders a single Diagnostic into a formatted string and prints it to stderr.
    pub fn render_stderr(&self, report: annotate_snippets::Report) {
        anstream::eprintln!("{}", self.inner.renderer.render(report));
    }

    /// Renders multiple Diagnostics into formatted strings and prints them to stderr.
    pub fn render(&self, reports: annotate_snippets::Report) {
        anstream::eprintln!("{}", self.inner.renderer.render(reports));
    }
}

#[salsa::accumulator]
#[derive(Debug)]
pub struct SalsaDiago(pub Group<'static>);

#[cfg(test)]
mod tests {

    use annotate_snippets::Patch;
    use anstyle::{AnsiColor, Effects, Style};

    use super::*;

    #[test]
    fn test_diagnostic_emitter() {
        const MAGENTA: Style = AnsiColor::Magenta.on_default().effects(Effects::BOLD);
        let message =
            format!("expected expression `let y = x + {MAGENTA}{{expr}}{MAGENTA:#} ;` found `;`",);

        let source = "let x = 5;\nlet y = x + ;\n";
        let file_name = "test.sc";

        let emitter = DiagnosticEmitter::new().with_auto_render(true);

        emitter.emit_err(
            Level::ERROR
                .primary_title("unexpected token found")
                .id("E0234")
                .element(
                    Snippet::source(source).path(file_name).annotation(
                        AnnotationKind::Primary
                            .span(22..25)
                            .label("expected expression here found `;` instead"),
                    ),
                )
                .element(Level::NOTE.message(&message))
                .element(
                    Snippet::source(source)
                        .path(file_name)
                        .patch(Patch::new(23..23, "<expr>")),
                ),
        );
    }

    #[test]
    fn emit_rendered_static_str_stays_borrowed() {
        let emitter = DiagnosticEmitter::new().with_render_on_drop(false);
        emitter.emit_rendered(Level::ERROR, "unexpected EOF");

        let guard = emitter.inner.errors.lock();
        assert_eq!(guard.len(), 1);
        assert!(matches!(
            &guard[0],
            StoredDiag::Rendered {
                text: Cow::Borrowed(_),
                ..
            }
        ));
    }

    #[test]
    fn emit_lazy_defers_render() {
        let emitter = DiagnosticEmitter::new().with_render_on_drop(false);
        emitter.emit_lazy(
            Level::ERROR,
            Level::ERROR
                .primary_title("boom")
                .element(Level::NOTE.message("detail")),
        );

        let guard = emitter.inner.errors.lock();
        assert!(matches!(&guard[0], StoredDiag::Lazy { .. }));
    }
}
