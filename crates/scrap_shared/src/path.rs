use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{
    NodeId,
    ident::{Ident, Symbol},
    pretty_print::PrettyPrint,
};

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct PathSegment {
    /// The identifier portion of this path segment.
    pub ident: Ident,
    /// The unique ID of this path segment.
    pub id: NodeId,
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::SalsaValue, serde::Serialize, serde::Deserialize,
)]
pub struct Path {
    pub span: Span,
    /// The segments in the path: the things separated by `::`.
    /// Global paths begin with `kw::PathRoot`.
    pub segments: ThinVec<PathSegment>,
}

impl PrettyPrint for Path {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, _indent: usize) -> std::fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }
            segment.ident.pretty_print_indent(f, 0)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let segment =
            salsa::with_attached_database(|db| self.to_string_db(db)).unwrap_or_else(|| {
                self.segments
                    .iter()
                    .map(|_| "<unknown>".to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            });

        write!(f, "{}", segment)
    }
}

impl Path {
    pub fn to_string_db(&self, _db: &dyn salsa::Database) -> String {
        let segments: Vec<String> = self
            .segments
            .iter()
            .map(|seg| seg.ident.name.text().to_string())
            .collect();
        segments.join("::")
    }

    pub fn from_ident(ident: Ident) -> Self {
        let id = ident.id;
        Path {
            span: ident.span,
            segments: ThinVec::from([PathSegment { ident, id }]),
        }
    }

    pub fn from_segments(segments: &[String]) -> Self {
        let mut path_segments = ThinVec::new();
        let mut start = 0;
        let mut end = 0;
        for segment in segments {
            end += segment.len();
            let ident = Ident {
                id: NodeId::dummy(),
                name: Symbol::new(segment),
                span: Span::new(start, end),
            };
            path_segments.push(PathSegment {
                ident,
                id: ident.id,
            });
            start = end + 2; // +2 for '::'
            end = start;
        }
        Path {
            span: Span::new(0, end - 2),
            segments: path_segments,
        }
    }

    pub fn from_segment(segment: &str) -> Self {
        let ident = Ident {
            id: NodeId::dummy(),
            name: Symbol::new(segment),
            span: Span::default(),
        };
        Self::from_ident(ident)
    }

    pub fn single_segment(&self) -> Option<&PathSegment> {
        if self.segments.len() == 1 {
            Some(&self.segments[0])
        } else {
            None
        }
    }

    pub fn extend(&self, ident: Ident) -> Self {
        let mut new_segments = self.segments.clone();
        new_segments.push(PathSegment {
            ident,
            id: ident.id,
        });
        Path {
            span: Span::new(self.span.start, ident.span.end),
            segments: new_segments,
        }
    }

    pub fn extend_segment(&self, segment: &str) -> Self {
        let ident = Ident {
            id: NodeId::dummy(),
            name: Symbol::new(segment),
            span: Span::default(),
        };
        self.extend(ident)
    }

    /// Compare two paths by text only, ignoring spans and NodeIds
    pub fn eq_text(&self, other: &Self) -> bool {
        if self.segments.len() != other.segments.len() {
            return false;
        }
        self.segments
            .iter()
            .zip(other.segments.iter())
            .all(|(a, b)| a.ident.name.text() == b.ident.name.text())
    }

    /// Compare two paths including spans (full structural equality)
    pub fn eq_with_span(&self, other: &Self) -> bool {
        self == other
    }
}

/// A wrapper around Path that provides equality and hashing based only on
/// the path text (segment names), ignoring spans and NodeIds.
/// This is used for interning ModuleId and TypeId.
#[derive(Debug, Clone, salsa::SalsaValue, serde::Serialize, serde::Deserialize)]
pub struct PathKey {
    pub path: Path,
}

impl PathKey {
    pub fn new(path: Path) -> Self {
        Self { path }
    }

    /// Compare including span information
    pub fn eq_with_span(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl std::hash::Hash for PathKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for segment in &self.path.segments {
            segment.ident.name.hash(state);
        }
    }
}

impl PartialEq for PathKey {
    fn eq(&self, other: &Self) -> bool {
        if self.path.segments.len() != other.path.segments.len() {
            return false;
        }
        self.path
            .segments
            .iter()
            .zip(other.path.segments.iter())
            .all(|(a, b)| a.ident.name == b.ident.name)
    }
}

impl Eq for PathKey {}

impl std::fmt::Display for PathKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}
