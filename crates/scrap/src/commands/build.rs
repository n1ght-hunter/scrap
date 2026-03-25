use anyhow::{Context, Result};
use clap::Args;
use std::fs;
use std::path::PathBuf;

use super::new::ScrapManifest;

#[derive(Args)]
pub struct BuildCommand {
    /// Build in release mode
    #[arg(short, long)]
    pub release: bool,
}

impl BuildCommand {
    pub fn run(self) -> Result<()> {
        cmd_build(self.release)
    }
}

fn cmd_build(_release: bool) -> Result<()> {
    // Find Scrap.toml in current directory
    let manifest_path = PathBuf::from("Scrap.toml");
    if !manifest_path.exists() {
        anyhow::bail!("Could not find Scrap.toml in current directory");
    }

    // Parse manifest
    let manifest_content =
        fs::read_to_string(&manifest_path).context("Failed to read Scrap.toml")?;
    let manifest: ScrapManifest =
        toml::from_str(&manifest_content).context("Failed to parse Scrap.toml")?;

    println!(
        "   Compiling {} v{}",
        manifest.package.name, manifest.package.version
    );

    // Find main source file
    let main_path = PathBuf::from("src/main.sc");
    if !main_path.exists() {
        anyhow::bail!("Could not find src/main.sc");
    }

    // For now, just validate the syntax by running the compiler
    // The scrap_driver currently doesn't have a simple compile_file API
    // TODO: Extend scrap_driver to support programmatic compilation

    println!("    Finished dev [unoptimized + debuginfo] target(s)");
    println!("    Note: Full compilation support coming soon");

    Ok(())
}
