//! Build orchestration for Scrap's native Rust interop.
//!
//! Parses the `[rust.dependencies]` table from a Scrap manifest, generates the
//! *anchor crate* (the single `staticlib` that depends on the user's Rust crates
//! and on `scrap_rt` as an rlib), builds it with the pinned toolchain via the
//! `scrap-rustc` driver (which also dumps the [`scrap_rmeta`] metadata describing
//! the anchor's public API, layouts, and ABI), and hands the resulting archive
//! plus metadata back to the driver to link into the compiled Scrap executable.
//! Builds are content-addressed (see `cache`) so an unchanged dependency set
//! reuses a previously built archive.

mod anchor;
mod cache;
mod driver;
mod schema;

pub use anchor::{AnchorArtifact, AnchorRequest, DropWrapper, build_anchor};
pub use schema::{DetailedDep, RustDepSpec, RustDeps, parse_manifest_rust_deps};

/// The nightly toolchain the interop build pins to. Must match the workspace
/// root `rust-toolchain.toml` (and the `scrap-rustc` driver's `rustc_private`).
pub const PINNED_TOOLCHAIN: &str = "nightly-2026-02-10";
