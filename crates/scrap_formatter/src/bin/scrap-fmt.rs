use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process;

/// Scrap language code formatter
#[derive(Parser, Debug)]
#[command(name = "scrap-fmt")]
#[command(about = "Scrap language code formatter", long_about = None)]
#[command(version)]
struct Args {
    /// Check if files are formatted without modifying them
    #[arg(long)]
    check: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Files to format
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn format_file(path: &PathBuf, check: bool, verbose: bool) -> Result<bool, String> {
    if verbose {
        println!("Processing: {}", path.display());
    }

    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    // For now, we'll use a simplified formatter
    // In a real implementation, you would:
    // 1. Create a Salsa database
    // 2. Lex the file
    // 3. Parse with Rowan parser
    // 4. Format using the pretty printer

    let config = scrap_formatter::FormatterConfig::default();
    let formatted = scrap_formatter::format_file(&source, &config);

    if check {
        if source != formatted {
            println!("{}: File is not formatted", path.display());
            return Ok(false);
        } else {
            if verbose {
                println!("{}: File is formatted correctly", path.display());
            }
            return Ok(true);
        }
    } else {
        if source != formatted {
            fs::write(path, &formatted)
                .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
            if verbose {
                println!("{}: Formatted", path.display());
            }
        } else {
            if verbose {
                println!("{}: No changes needed", path.display());
            }
        }
        return Ok(true);
    }
}

fn main() {
    let args = Args::parse();

    let mut all_formatted = true;

    for file in &args.files {
        match format_file(file, args.check, args.verbose) {
            Ok(formatted) => {
                if !formatted {
                    all_formatted = false;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                all_formatted = false;
            }
        }
    }

    if args.check && !all_formatted {
        eprintln!("\nSome files are not formatted. Run without --check to format them.");
        process::exit(1);
    }

    if args.verbose {
        println!("\nDone!");
    }
}
