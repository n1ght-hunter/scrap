use core::fmt;

use ariadne::Color;

/// A type that defines the kind of report being produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportKind {
    /// The report is an error and indicates a critical problem that prevents the program performing the requested
    /// action.
    Error,
    /// The report is a warning and indicates a likely problem, but not to the extent that the requested action cannot
    /// be performed.
    Warning,
    /// The report is advice to the user about a potential anti-pattern of other benign issues.
    Advice,
    /// The report is of a kind not built into Ariadne.
    Custom(&'static str, Color),
}

impl ReportKind {
    /// Create a custom report kind.
    pub fn custom(msg: &'static str, color: Color) -> Self {
        ReportKind::Custom(msg, color)
    }

    pub fn color(&self) -> Color {
        match self {
            ReportKind::Error => Color::Red,
            ReportKind::Warning => Color::Yellow,
            ReportKind::Advice => Color::Cyan,
            ReportKind::Custom(_, color) => *color,
        }
    }
}

impl fmt::Display for ReportKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReportKind::Error => write!(f, "Error"),
            ReportKind::Warning => write!(f, "Warning"),
            ReportKind::Advice => write!(f, "Advice"),
            ReportKind::Custom(s, _) => write!(f, "{}", s),
        }
    }
}

impl From<ReportKind> for ariadne::ReportKind<'_> {
    fn from(kind: ReportKind) -> Self {
        match kind {
            ReportKind::Error => ariadne::ReportKind::Error,
            ReportKind::Warning => ariadne::ReportKind::Warning,
            ReportKind::Advice => ariadne::ReportKind::Advice,
            ReportKind::Custom(msg, color) => ariadne::ReportKind::Custom(msg, color),
        }
    }
}