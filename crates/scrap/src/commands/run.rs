use anyhow::Result;
use clap::Args;

use super::build::BuildCommand;

#[derive(Args)]
pub struct RunCommand {
    /// Build in release mode
    #[arg(short, long)]
    pub release: bool,
    /// Arguments to pass to the program
    #[arg(last = true)]
    pub args: Vec<String>,
}

impl RunCommand {
    pub fn run(self) -> Result<()> {
        cmd_run(self.release, self.args)
    }
}

fn cmd_run(release: bool, _args: Vec<String>) -> Result<()> {
    // First build the project
    BuildCommand { release }.run()?;

    println!("     Note: Execution support coming soon");
    println!("     The scrap driver currently only supports syntax validation");

    Ok(())
}
