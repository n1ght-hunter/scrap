use scrap_shared::pretty_print::PrettyPrint;
use scrap_span::Span;
use thin_vec::ThinVec;

use crate::{ident::Ident, node_id::NodeId};

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct PathSegment<'db> {
    /// The identifier portion of this path segment.
    pub ident: Ident<'db>,
    /// The unique ID of this path segment.
    pub id: NodeId,
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Path<'db> {
    pub span: Span<'db>,
    /// The segments in the path: the things separated by `::`.
    /// Global paths begin with `kw::PathRoot`.
    pub segments: ThinVec<PathSegment<'db>>,
}

impl<'db> PrettyPrint for Path<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }
            segment.ident.pretty_print(f)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Path<'_> {
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

impl<'db> Path<'db> {
    pub fn to_string_db(&self, db: &'db dyn salsa::Database) -> String {
        let segments: Vec<String> = self
            .segments
            .iter()
            .map(|seg| seg.ident.name.text(db).to_string())
            .collect();
        segments.join("::")
    }

    pub fn from_ident(ident: Ident<'db>) -> Self {
        let id = ident.id;
        Path {
            span: ident.span,
            segments: ThinVec::from([PathSegment { ident, id }]),
        }
    }

    pub fn from_segments(db: &'db dyn scrap_shared::Db, segments: &[String]) -> Self {
        let mut path_segments = ThinVec::new();
        let mut start = 0;
        let mut end = 0;
        for segment in segments {
            end += segment.len();
            let ident = Ident {
                id: NodeId::dummy(),
                name: scrap_span::Symbol::new(db, segment),
                span: Span::new(db, start, end),
            };
            path_segments.push(PathSegment {
                ident,
                id: ident.id,
            });
            start = end + 2; // +2 for '::'
            end = start;
        }
        Path {
            span: Span::new(db, 0, end - 2), // -2 to remove last '::'
            segments: path_segments,
        }
    }

    pub fn from_segment(db: &'db dyn scrap_shared::Db, segment: &str) -> Self {
        let ident = Ident {
            id: NodeId::dummy(),
            name: scrap_span::Symbol::new(db, segment),
            span: Span::new_default(db),
        };
        Self::from_ident(ident)
    }

    pub fn single_segment(&self) -> Option<&PathSegment<'db>> {
        if self.segments.len() == 1 {
            Some(&self.segments[0])
        } else {
            None
        }
    }

    pub fn extend(&self, db: &'db dyn scrap_shared::Db, ident: Ident<'db>) -> Self {
        let mut new_segments = self.segments.clone();
        new_segments.push(PathSegment {
            ident: ident,
            id: ident.id,
        });
        Path {
            span: Span::new(db, self.span.start(db), ident.span.end(db)),
            segments: new_segments,
        }
    }

    pub fn extend_segment(&self, db: &'db dyn scrap_shared::Db, segment: &str) -> Self {
        let ident = Ident {
            id: NodeId::dummy(),
            name: scrap_span::Symbol::new(db, segment),
            span: Span::new_default(db),
        };
        self.extend(db, ident)
    }

    pub fn to_key(&self) -> PathKey<'db> {
        PathKey(self.segments.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathKey<'db>(ThinVec<PathSegment<'db>>);

impl radix_trie::TrieKey for PathKey<'_> {
    fn encode_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.0.len() * std::mem::size_of::<u64>());
        for segment in self.0.iter() {
            let name = segment.ident.name.as_bits().to_le_bytes();
            bytes.extend_from_slice(&name);
        }
        bytes
    }
}
