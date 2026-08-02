//! Locating the `scrap-rustc` metadata driver.
//!
//! The driver is an *artifact dependency* (`-Zbindeps`): cargo builds it as part
//! of building this crate and hands us the path through
//! `CARGO_BIN_FILE_SCRAP_RUSTC`. That makes it a real edge in the build graph,
//! which is what we want — the driver depends on `scrap_rmeta`, so a
//! `SCHEMA_VERSION` bump rebuilds it automatically. The previous design shelled
//! out to `cargo build` on an anchor cache *miss*, so a bump with a warm cache
//! left a stale binary emitting the old schema while `read_metadata` rejected
//! every dump it produced.
//!
//! It links the pinned nightly's `rustc_private`, so it stays in a detached
//! workspace under `tools/scrap-rustc` (its own `rust-toolchain.toml` adds
//! `rustc-dev`) and the dependency is behind the off-by-default
//! `interop-driver` feature.

use std::path::PathBuf;

/// Path to the `scrap-rustc` binary cargo built for us.
#[cfg(feature = "interop-driver")]
pub(crate) fn driver_path() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(env!("CARGO_BIN_FILE_SCRAP_RUSTC")))
}

/// Without the `interop-driver` feature there is no driver to run, so an anchor
/// build cannot proceed.
#[cfg(not(feature = "interop-driver"))]
pub(crate) fn driver_path() -> anyhow::Result<PathBuf> {
    anyhow::bail!(
        "this build of scrapc has Rust interop disabled, but the manifest declares \
         [rust.dependencies]; rebuild with `--features interop-driver` (requires the \
         `rustc-dev` component: `rustup component add rustc-dev`)"
    )
}
