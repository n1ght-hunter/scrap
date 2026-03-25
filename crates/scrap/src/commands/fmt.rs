use anyhow::{Context, Result};
use clap::Args;
use std::fs;
use std::path::PathBuf;

#[derive(Args)]
pub struct FmtCommand {
    /// Path to file or directory to format
    pub path: Option<PathBuf>,
    /// Check if files are formatted without writing
    #[arg(short, long)]
    pub check: bool,
}

impl FmtCommand {
    pub fn run(self) -> Result<()> {
        cmd_fmt(self.path, self.check)
    }
}

fn cmd_fmt(path: Option<PathBuf>, check: bool) -> Result<()> {
    let target_path = path.unwrap_or_else(|| PathBuf::from("src"));

    if !target_path.exists() {
        anyhow::bail!("Path '{}' does not exist", target_path.display());
    }

    let config = scrap_formatter::FormatterConfig::default();

    if target_path.is_file() {
        format_file_path(&target_path, &config, check)?;
    } else if target_path.is_dir() {
        format_directory(&target_path, &config, check)?;
    } else {
        anyhow::bail!(
            "Path '{}' is neither a file nor directory",
            target_path.display()
        );
    }

    if check {
        println!("All files are formatted correctly");
    }

    Ok(())
}

fn format_file_path(
    path: &PathBuf,
    config: &scrap_formatter::FormatterConfig,
    check: bool,
) -> Result<()> {
    if path.extension().and_then(|s| s.to_str()) != Some("sc") {
        return Ok(()); // Skip non-.sc files
    }

    let source = fs::read_to_string(path).context(format!("Failed to read {}", path.display()))?;

    let formatted = scrap_formatter::format_file(&source, config);

    if check {
        if source != formatted {
            anyhow::bail!("File {} is not formatted correctly", path.display());
        }
        println!("✓ {}", path.display());
    } else {
        fs::write(path, &formatted).context(format!("Failed to write {}", path.display()))?;
        println!("Formatted {}", path.display());
    }

    Ok(())
}

fn format_directory(
    dir: &PathBuf,
    config: &scrap_formatter::FormatterConfig,
    check: bool,
) -> Result<()> {
    for entry in fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            format_file_path(&path, config, check)?;
        } else if path.is_dir() {
            format_directory(&path, config, check)?;
        }
    }

    Ok(())
}
