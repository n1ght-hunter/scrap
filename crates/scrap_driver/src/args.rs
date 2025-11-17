use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The entry source file to compile.
    #[arg(required = true, value_parser = clap::value_parser!(PathBuf))]
    pub entry_source_file: PathBuf,

    /// The source files to compile.
    #[arg(value_parser = clap::value_parser!(PathBuf), long, short = 'i')]
    pub source_files: Vec<PathBuf>,

    /// Set the name of the output crate.
    #[arg(long)]
    pub crate_name: String,

    /// Specify the type of crate to build.
    #[arg(long)]
    pub crate_type: CrateType,

    #[arg(long)]
    pub cache: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    pub clear_cache: bool,

    /// Add a directory to the library search path.
    #[arg(short = 'L', long, value_name = "PATH")]
    pub library_path: Vec<PathBuf>,

    /// Specify an external crate to link against.
    #[arg(long, value_name = "CRATENAME=PATH")]
    pub extern_crate: Vec<String>,
    /// The type of output to generate
    #[clap(long = "unpretty-out")]
    pub unpretty_out: Option<UnPrettyOut>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum)]
pub enum UnPrettyOut {
    /// Generate and print the abstract syntax tree (AST)
    Ast,
    /// Generate Scrap IR
    SIR,
    /// Generate unoptimized CraneLift IR
    CLIR,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum)]
pub enum CrateType {
    Bin,
    Lib,
    Rlib,
    Dylib,
    Staticlib,
}
