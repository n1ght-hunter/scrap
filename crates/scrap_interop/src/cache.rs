//! Content-addressed cache key for an anchor build.
//!
//! The key folds everything that changes the produced archive: the dependency
//! set, target triple, toolchain channel, profile, the metadata schema version,
//! and a generator version that is bumped whenever the anchor-generation logic
//! itself changes. An unchanged key lets the driver reuse a previously built
//! archive instead of re-running cargo.
//!
//! It does *not* hash extractor logic or path-dependency source content, so
//! editing either still needs a manual `target/scrap/anchor` clear (or a
//! `GENERATOR_VERSION` bump).

use std::hash::{Hash, Hasher};

use crate::anchor::DropWrapper;
use crate::schema::{RustDepSpec, RustDeps};

/// Bump whenever the generated anchor files or build command change shape, so
/// stale caches from an older scrapc are invalidated.
const GENERATOR_VERSION: u32 = 1;

/// Compute the stable cache hash for an anchor build, as a hex string. The drop
/// wrapper set is included so the metadata-only build and the wrapper build are
/// distinct cache entries.
pub(crate) fn cache_key(
    deps: &RustDeps,
    target: &str,
    toolchain_channel: &str,
    release: bool,
    drop_wrappers: &[DropWrapper],
) -> String {
    let mut hasher = wyhash::WyHash::with_seed(0);
    GENERATOR_VERSION.hash(&mut hasher);
    // A schema bump must bust the archive+metadata cache so a fresh dump is
    // produced rather than reusing one the current scrapc would reject.
    scrap_rmeta::SCHEMA_VERSION.hash(&mut hasher);
    target.hash(&mut hasher);
    toolchain_channel.hash(&mut hasher);
    release.hash(&mut hasher);

    // BTreeMap iteration is ordered, so this is deterministic.
    for (name, spec) in &deps.0 {
        name.hash(&mut hasher);
        hash_spec(spec, &mut hasher);
    }
    // Drop wrappers are produced in a deterministic order by the driver.
    for w in drop_wrappers {
        w.sanitized.hash(&mut hasher);
        w.full_path.hash(&mut hasher);
    }

    format!("{:016x}", hasher.finish())
}

fn hash_spec(spec: &RustDepSpec, hasher: &mut impl Hasher) {
    match spec {
        RustDepSpec::Version(v) => {
            0u8.hash(hasher);
            v.hash(hasher);
        }
        RustDepSpec::Detailed(d) => {
            1u8.hash(hasher);
            d.version.hash(hasher);
            d.features.hash(hasher);
            d.default_features.hash(hasher);
            d.path.hash(hasher);
            d.git.hash(hasher);
            d.branch.hash(hasher);
            d.tag.hash(hasher);
            d.rev.hash(hasher);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::DetailedDep;
    use std::collections::BTreeMap;

    fn deps_with(entries: &[(&str, RustDepSpec)]) -> RustDeps {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        RustDeps(m)
    }

    #[test]
    fn key_is_stable_and_order_independent() {
        let a = deps_with(&[
            ("regex", RustDepSpec::Version("1".into())),
            ("serde", RustDepSpec::Version("1".into())),
        ]);
        let b = deps_with(&[
            ("serde", RustDepSpec::Version("1".into())),
            ("regex", RustDepSpec::Version("1".into())),
        ]);
        let ka = cache_key(&a, "x86_64-pc-windows-msvc", "nightly", false, &[]);
        let kb = cache_key(&b, "x86_64-pc-windows-msvc", "nightly", false, &[]);
        assert_eq!(ka, kb);
        assert_eq!(
            ka,
            cache_key(&a, "x86_64-pc-windows-msvc", "nightly", false, &[])
        );
    }

    #[test]
    fn key_changes_with_inputs() {
        let base = deps_with(&[("regex", RustDepSpec::Version("1".into()))]);
        let key = cache_key(&base, "x86_64-pc-windows-msvc", "nightly", false, &[]);
        assert_ne!(
            key,
            cache_key(&base, "x86_64-unknown-linux-gnu", "nightly", false, &[])
        );
        assert_ne!(
            key,
            cache_key(&base, "x86_64-pc-windows-msvc", "nightly", true, &[])
        );
        let changed = deps_with(&[(
            "regex",
            RustDepSpec::Detailed(DetailedDep {
                version: Some("1".into()),
                ..Default::default()
            }),
        )]);
        assert_ne!(
            key,
            cache_key(&changed, "x86_64-pc-windows-msvc", "nightly", false, &[])
        );
    }
}
