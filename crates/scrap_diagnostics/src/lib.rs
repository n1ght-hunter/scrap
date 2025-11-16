//! Defines the structures and rendering logic for compiler diagnostics.
//! This is the "presentation layer" for errors.

pub use annotate_snippets::{self, Annotation, AnnotationKind, Level, Snippet};
pub use anstream;
pub use anstyle;

use annotate_snippets::{Group, Renderer, Report, renderer::DecorStyle};

pub struct DiagnosticEmitter<'a> {
    diagnostics: Vec<Group<'a>>,
    renderer: Renderer,
    /// Whether diagnostics should be automatically rendered when emitted.
    auto_render: bool,
}

impl Default for DiagnosticEmitter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DiagnosticEmitter<'a> {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            renderer: Renderer::styled().decor_style(DecorStyle::Unicode),
            auto_render: false,
        }
    }

    pub fn with_auto_render(mut self, auto_render: bool) -> Self {
        self.auto_render = auto_render;
        self
    }

    pub fn emit(&mut self, diag: Group<'a>) {
        if self.auto_render {
            self.render(&[diag.clone()]);
        }
        self.diagnostics.push(diag);
    }

    pub fn render_all(&self) {
        self.render(&self.diagnostics);
    }

    pub fn append(&mut self, mut other: DiagnosticEmitter<'a>) {
        self.diagnostics.append(&mut other.diagnostics);
    }

    pub fn render_single(&self, diag: Group<'a>) {
        self.render(&[diag]);
    }

    /// Renders a single Diagnostic into a formatted string.
    pub fn render(&self, groups: impl IntoDiagnosticGroup<'a>) {
        anstream::println!("{}", self.renderer.render(groups.into_diagnostic_group()));
    }
}

mod sealed {
    pub trait Sealed {}
}

pub trait IntoDiagnosticGroup<'a>: sealed::Sealed {
    fn into_diagnostic_group(self) -> Report<'a>;
}

impl<'a> sealed::Sealed for &'a [Group<'a>] {}

impl<'a> IntoDiagnosticGroup<'a> for &'a [Group<'a>] {
    fn into_diagnostic_group(self) -> Report<'a> {
        self
    }
}

impl<'a, T> sealed::Sealed for &'a T where T: AsRef<[Group<'a>]> {}

impl<'a, T> IntoDiagnosticGroup<'a> for &'a T
where
    T: AsRef<[Group<'a>]>,
{
    fn into_diagnostic_group(self) -> Report<'a> {
        self.as_ref()
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

        let mut emitter = DiagnosticEmitter::new().with_auto_render(true);

        emitter.emit(
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
                        .patch(Patch::new(23..22, "<expr>")),
                ),
        );
    }
}
