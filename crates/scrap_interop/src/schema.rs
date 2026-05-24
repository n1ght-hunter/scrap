//! The `[rust.dependencies]` manifest table and its rendering into a Cargo
//! `[dependencies]` section for the generated anchor crate.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;

/// The set of Rust crate dependencies declared in a Scrap manifest.
///
/// Backed by a `BTreeMap` so iteration order is deterministic — the cache key
/// hashes this and must be stable across runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RustDeps(pub BTreeMap<String, RustDepSpec>);

impl RustDeps {
    /// Whether any Rust dependency is declared. Empty → the driver skips the
    /// anchor build entirely and keeps the plain `scrap_rt.lib` link path.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A single dependency entry: either a bare version string or a detailed table,
/// mirroring Cargo's own dependency syntax.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum RustDepSpec {
    Version(String),
    Detailed(DetailedDep),
}

/// The detailed (`{ version = .., features = [..] }`) form of a dependency.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetailedDep {
    pub version: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(rename = "default-features")]
    pub default_features: Option<bool>,
    pub path: Option<PathBuf>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
}

#[derive(Deserialize)]
struct ManifestRoot {
    rust: Option<RustSection>,
}

#[derive(Deserialize)]
struct RustSection {
    #[serde(default)]
    dependencies: BTreeMap<String, RustDepSpec>,
}

/// Parse the `[rust.dependencies]` table from a Scrap manifest. A missing file,
/// missing `[rust]` section, or missing `dependencies` all yield empty deps;
/// the `[package]` section and anything else are ignored.
pub fn parse_manifest_rust_deps(manifest: &Path) -> anyhow::Result<RustDeps> {
    if !manifest.exists() {
        return Ok(RustDeps::default());
    }
    let text = std::fs::read_to_string(manifest)
        .with_context(|| format!("failed to read manifest {}", manifest.display()))?;
    let root: ManifestRoot = toml::from_str(&text)
        .with_context(|| format!("failed to parse manifest {}", manifest.display()))?;
    Ok(RustDeps(
        root.rust.map(|r| r.dependencies).unwrap_or_default(),
    ))
}

impl RustDepSpec {
    /// Render this spec as the value side of a Cargo `[dependencies]` entry
    /// (everything after `name = `). Paths are emitted as TOML *literal* strings
    /// (single-quoted) so Windows backslashes are not treated as escapes.
    fn render_value(&self) -> String {
        match self {
            RustDepSpec::Version(v) => format!("\"{v}\""),
            RustDepSpec::Detailed(d) => {
                let mut parts: Vec<String> = Vec::new();
                if let Some(v) = &d.version {
                    parts.push(format!("version = \"{v}\""));
                }
                if !d.features.is_empty() {
                    let feats = d
                        .features
                        .iter()
                        .map(|f| format!("\"{f}\""))
                        .collect::<Vec<_>>()
                        .join(", ");
                    parts.push(format!("features = [{feats}]"));
                }
                if let Some(df) = d.default_features {
                    parts.push(format!("default-features = {df}"));
                }
                if let Some(p) = &d.path {
                    parts.push(format!("path = '{}'", p.display()));
                }
                if let Some(g) = &d.git {
                    parts.push(format!("git = \"{g}\""));
                }
                if let Some(b) = &d.branch {
                    parts.push(format!("branch = \"{b}\""));
                }
                if let Some(t) = &d.tag {
                    parts.push(format!("tag = \"{t}\""));
                }
                if let Some(r) = &d.rev {
                    parts.push(format!("rev = \"{r}\""));
                }
                format!("{{ {} }}", parts.join(", "))
            }
        }
    }

    /// Render a full `name = value` Cargo dependency line.
    pub(crate) fn render_line(&self, name: &str) -> String {
        format!("{name} = {}", self.render_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_and_detailed_forms() {
        let toml = r#"
[package]
name = "demo"
version = "0.1.0"

[rust.dependencies]
regex = "1"
serde = { version = "1", features = ["derive"], default-features = false }
mylib = { path = "../mylib" }
"#;
        let dir = std::env::temp_dir().join("scrap_interop_test_manifest");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Scrap.toml");
        std::fs::write(&path, toml).unwrap();

        let deps = parse_manifest_rust_deps(&path).unwrap();
        assert_eq!(deps.0.len(), 3);
        assert_eq!(
            deps.0.get("regex"),
            Some(&RustDepSpec::Version("1".to_string()))
        );
        match deps.0.get("serde").unwrap() {
            RustDepSpec::Detailed(d) => {
                assert_eq!(d.version.as_deref(), Some("1"));
                assert_eq!(d.features, vec!["derive".to_string()]);
                assert_eq!(d.default_features, Some(false));
            }
            other => panic!("expected detailed, got {other:?}"),
        }
    }

    #[test]
    fn missing_file_is_empty() {
        let deps = parse_manifest_rust_deps(Path::new("does/not/exist/Scrap.toml")).unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn renders_dependency_lines() {
        assert_eq!(
            RustDepSpec::Version("1.0".into()).render_line("regex"),
            r#"regex = "1.0""#
        );
        let d = DetailedDep {
            version: Some("1".into()),
            features: vec!["derive".into()],
            default_features: Some(false),
            ..Default::default()
        };
        assert_eq!(
            RustDepSpec::Detailed(d).render_line("serde"),
            r#"serde = { version = "1", features = ["derive"], default-features = false }"#
        );
    }
}
