//! Locating and building the `scrap-rustc` metadata driver.
//!
//! The driver links the pinned nightly's `rustc_private`, so it lives in a
//! detached workspace under `tools/scrap-rustc` and is built on demand (its own
//! `rust-toolchain.toml` selects the right toolchain). We always invoke `cargo
//! build` (cheap when up to date) rather than skipping on a present binary: the
//! driver depends on `scrap_rmeta`, so a `SCHEMA_VERSION` bump must rebuild it
//! to emit the new schema — otherwise a stale binary would keep emitting the old
//! schema and `read_metadata` would reject every dump in a permanent loop. This
//! only runs on an anchor cache miss, where a cargo no-op check is negligible.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};

fn driver_bin_name() -> &'static str {
    if cfg!(windows) {
        "scrap-rustc.exe"
    } else {
        "scrap-rustc"
    }
}

/// Ensure the `scrap-rustc` binary exists (building it if needed) and return its
/// path. `crate_dir` is the `tools/scrap-rustc` directory.
pub(crate) fn ensure_driver(crate_dir: &Path) -> anyhow::Result<PathBuf> {
    let bin = crate_dir
        .join("target")
        .join("debug")
        .join(driver_bin_name());

    let status = Command::new("cargo")
        // Run in the driver dir so its rust-toolchain.toml (pinned nightly +
        // rustc-dev) is selected.
        .current_dir(crate_dir)
        .arg("build")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .status()
        .context("failed to spawn cargo to build the scrap-rustc driver")?;
    if !status.success() {
        bail!("building the scrap-rustc driver failed");
    }
    if !bin.exists() {
        bail!(
            "scrap-rustc built but its binary was not found at {}",
            bin.display()
        );
    }
    Ok(bin)
}
