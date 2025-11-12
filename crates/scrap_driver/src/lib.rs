mod args;

use std::{ffi::OsString, path::PathBuf};

use clap::{Parser, ValueEnum};
use scrap_ast::{Can, module::Module};
use scrap_parser::TokenStream;

#[salsa::input(debug)]
struct InputFile {
    pub path: PathBuf,
    #[returns(ref)]
    pub content: String,
}

#[salsa::tracked]
struct LexedFile<'db> {
    pub tokens: TokenStream,
}

#[salsa::tracked]
struct ParsedFile<'db> {
    pub ast: AstRoot,
}

enum AstRoot {
    Can(Can),
    Module(Module),
}

pub fn run_complier<I, T>(itr: I)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = args::Args::parse_from(itr);

    let ast = scrap_parser::parse_files(files)?;

    if let Some(UnPrettyOut::Ast) = args.unpretty_out {
        println!("{:#?}", ast);
        return Ok(());
    }

    let mir = scrap_ir::mir_builder::MirBuilder::new().lower_can(ast)?;

    if let Some(UnPrettyOut::Mir) = args.unpretty_out {
        println!("{:#?}", mir);
        return Ok(());
    }
}
