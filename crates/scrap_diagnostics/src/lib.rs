//! Defines the structures and rendering logic for compiler diagnostics.
//! This is the "presentation layer" for errors.

use std::sync::{Arc, atomic::AtomicBool};

pub use annotate_snippets::{self, Annotation, AnnotationKind, Level, Snippet};
pub use anstream;
pub use anstyle;

use annotate_snippets::{Group, Renderer, Report, renderer::DecorStyle};
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use scrap_errors::ErrorGuaranteed;

#[derive(Clone)]
pub struct DiagnosticEmitter<'a> {
    diagnostics: Arc<DiagnosticInner<'a>>,
    renderer: Renderer,
    /// Whether diagnostics should be automatically rendered when emitted.
    auto_render: bool,
    /// Whether all diagnostics should be rendered when the emitter is dropped.
    /// This will use the Arc to count references, so it only renders when the last reference is dropped.
    render_on_drop: bool,
}

impl<'a> Default for DiagnosticEmitter<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DiagnosticEmitter<'_> {
    fn drop(&mut self) {
        if self.render_on_drop && Arc::strong_count(&self.diagnostics) == 1 {
            self.render_all();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Emitted {
    Yes,
    No,
}

#[derive(Default, Debug)]
struct DiagnosticInner<'a> {
    errors: parking_lot::Mutex<Vec<(Emitted, Group<'a>)>>,
    warnings: parking_lot::Mutex<Vec<(Emitted, Group<'a>)>>,
    others: parking_lot::Mutex<Vec<(Emitted, Group<'a>)>>,
}

impl<'a> DiagnosticInner<'a> {
    fn push(&self, level: Level<'_>, emitted: Emitted, diag: Group<'a>) {
        match level {
            Level::ERROR => {
                self.errors.lock().push((emitted, diag));
            }
            Level::WARNING => {
                self.warnings.lock().push((emitted, diag));
            }
            _ => {
                self.others.lock().push((emitted, diag));
            }
        }
    }

    fn has_errors(&self) -> bool {
        !self.errors.lock().is_empty()
    }

    fn has_warnings(&self) -> bool {
        !self.warnings.lock().is_empty()
    }

    fn has_unrendered(&self) -> bool {
        let check = |input: &parking_lot::Mutex<Vec<(Emitted, Group<'a>)>>| {
            let guard = input.lock();
            guard.par_iter().any(|(emitted, _)| *emitted == Emitted::No)
        };
        let result = AtomicBool::new(false);
        rayon::scope(|s| {
            s.spawn(|_| {
                if check(&self.errors) {
                    result.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            });
            s.spawn(|_| {
                if check(&self.warnings) {
                    result.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            });
            s.spawn(|_| {
                if check(&self.others) {
                    result.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            });
        });
        result.load(std::sync::atomic::Ordering::Relaxed)
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

    fn all_non_rendered(&self, render: impl Fn(Level, Report) + Sync + Send) {
        let render = |level: Level, input: &parking_lot::Mutex<Vec<(Emitted, Group<'a>)>>| {
            let mut guard = input.lock();
            let report = guard
                .par_iter_mut()
                .filter_map(|(emitted, diag)| {
                    if *emitted == Emitted::No {
                        *emitted = Emitted::Yes;
                        Some(diag.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if !report.is_empty() {
                render(level, &report);
            }
        };
        rayon::scope(|s| {
            s.spawn(|_| render(Level::ERROR, &self.errors));
            s.spawn(|_| render(Level::WARNING, &self.warnings));
            // info is used for "other" diagnostics
            s.spawn(|_| render(Level::INFO, &self.others));
        });
    }
}

impl<'a> DiagnosticEmitter<'a> {
    pub fn new() -> Self {
        Self {
            diagnostics: Arc::new(DiagnosticInner::default()),
            renderer: Renderer::styled().decor_style(DecorStyle::Unicode),
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
}

impl<'a> DiagnosticEmitter<'a> {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    pub fn has_warnings(&self) -> bool {
        self.diagnostics.has_warnings()
    }

    pub fn has_unrendered(&self) -> bool {
        self.diagnostics.has_unrendered()
    }

    pub fn clear(&self) {
        self.diagnostics.clear();
    }

    /// Returns the counts of (errors, warnings, others)
    pub fn counts(&self) -> (usize, usize, usize) {
        self.diagnostics.counts()
    }

    pub fn emit_err(&self, diag: Group<'a>) -> ErrorGuaranteed {
        self.emit(Level::ERROR, diag);
        #[allow(deprecated)]
        ErrorGuaranteed::unchecked_error_guaranteed()
    }

    pub fn emit(&self, level: Level<'_>, diag: Group<'a>) {
        let mut emitted = Emitted::No;
        if self.auto_render {
            emitted = Emitted::Yes;
            self.render(std::slice::from_ref(&diag));
        }
        self.diagnostics.push(level, emitted, diag);
    }

    pub fn render_all(&self) {
        self.diagnostics
            .all_non_rendered(|level, report| match level {
                Level::ERROR | Level::WARNING => self.render(report),
                _ => self.render_stderr(report),
            });
    }

    /// Renders a single Diagnostic into a formatted string and prints it to stderr.
    pub fn render_stderr(&self, report: Report) {
        anstream::eprintln!("{}", self.renderer.render(report));
    }

    /// Renders multiple Diagnostics into formatted strings and prints them to stderr.
    pub fn render(&self, reports: Report) {
        anstream::eprintln!("{}", self.renderer.render(reports));
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
                .element(Level::NOTE.message(message))
                .element(
                    Snippet::source(source)
                        .path(file_name)
                        .patch(Patch::new(23..23, "<expr>")),
                ),
        );
    }
}
