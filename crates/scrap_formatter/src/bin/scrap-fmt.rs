use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

struct Args {
    files: Vec<PathBuf>,
    check: bool,
    verbose: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut files = Vec::new();
    let mut check = false;
    let mut verbose = false;

    let args: Vec<_> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return Err("No files specified. Usage: scrap-fmt [OPTIONS] <files>...".to_string());
    }

    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            "--verbose" | "-v" => verbose = true,
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            _ => {
                if arg.starts_with("--") {
                    return Err(format!("Unknown option: {}", arg));
                }
                files.push(PathBuf::from(arg));
            }
        }
    }

    if files.is_empty() {
        return Err("No files specified".to_string());
    }

    Ok(Args {
        files,
        check,
        verbose,
    })
}

fn print_help() {
    println!("scrap-fmt - Scrap language code formatter");
    println!();
    println!("USAGE:");
    println!("    scrap-fmt [OPTIONS] <files>...");
    println!();
    println!("OPTIONS:");
    println!("    --check        Check if files are formatted without modifying them");
    println!("    --verbose, -v  Enable verbose output");
    println!("    --help, -h     Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    scrap-fmt main.sc");
    println!("    scrap-fmt --check src/*.sc");
    println!("    scrap-fmt --verbose lib.sc");
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
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            print_help();
            process::exit(1);
        }
    };

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
