mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{BuildCommand, FmtCommand, NewCommand, RunCommand};

#[derive(Parser)]
#[command(name = "scrap")]
#[command(about = "Scrap package manager - like cargo for rustc, scrap is for scrapc", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Scrap project
    New(NewCommand),
    /// Build the current project
    Build(BuildCommand),
    /// Build and run the current project
    Run(RunCommand),
    /// Format Scrap source files
    Fmt(FmtCommand),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New(cmd) => cmd.run(),
        Commands::Build(cmd) => cmd.run(),
        Commands::Run(cmd) => cmd.run(),
        Commands::Fmt(cmd) => cmd.run(),
    }
}
