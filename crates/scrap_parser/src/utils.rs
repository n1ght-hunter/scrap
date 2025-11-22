use scrap_diagnostics::annotate_snippets::Group;

pub fn render(report: &[Group]) -> ! {
    scrap_diagnostics::DiagnosticEmitter::new().render(report);
    scrap_errors::FatalError.raise()
}

pub trait ExtendRes<T> {
    fn should_panic(self) -> T;
}

impl<'a, T> ExtendRes<T> for crate::PResult<'a, T> {
    fn should_panic(self) -> T {
        match self {
            Ok(v) => v,
            Err(_) => scrap_errors::FatalError.raise(),
        }
    }
}
