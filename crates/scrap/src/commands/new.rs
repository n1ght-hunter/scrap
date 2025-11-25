use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapManifest {
    pub package: PackageInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

#[derive(Args)]
pub struct NewCommand {
    /// Name of the project
    pub name: String,
    /// Directory to create the project in (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

impl NewCommand {
    pub fn run(self) -> Result<()> {
        cmd_new(self.name, self.path)
    }
}

fn cmd_new(name: String, path: Option<PathBuf>) -> Result<()> {
    let base_path = path.unwrap_or_else(|| PathBuf::from("."));
    let project_path = base_path.join(&name);

    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", project_path.display());
    }

    println!("Creating new Scrap project: {}", name);

    // Create project directory structure
    fs::create_dir_all(&project_path)
        .context("Failed to create project directory")?;

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir)
        .context("Failed to create src directory")?;

    // Create Scrap.toml manifest
    let manifest = ScrapManifest {
        package: PackageInfo {
            name: name.clone(),
            version: "0.1.0".to_string(),
        },
    };

    let manifest_toml = toml::to_string_pretty(&manifest)
        .context("Failed to serialize manifest")?;

    fs::write(project_path.join("Scrap.toml"), manifest_toml)
        .context("Failed to write Scrap.toml")?;

    // Create main.sc
    let main_content = r#"fn main() {
    print("Hello, world!");
}
"#;
    fs::write(src_dir.join("main.sc"), main_content)
        .context("Failed to write main.sc")?;

    println!("     Created binary package `{}`", name);

    Ok(())
}
