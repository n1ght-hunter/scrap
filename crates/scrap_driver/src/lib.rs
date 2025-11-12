mod args;

use std::ffi::OsString;

use args::UnPrettyOut;
use clap::Parser;

#[salsa::tracked(debug)]
struct TrackedArgs<'db> {
    pub args: args::Args,
}

pub fn run_complier<I, T>(itr: I)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = args::Args::parse_from(itr);
    let db = scrap_shared::salsa::ScrapDb::default();

    let input_path = scrap_shared::salsa::InputPath::new(&db, args.source_file.clone());
    let input_file = scrap_shared::salsa::load_file(&db, input_path);
    let lexed_tokens = scrap_lexer::lex_file(&db, input_file);
    let parsed_file = scrap_parser::parse_tokens(&db, input_file, lexed_tokens, true);

    if let Some(UnPrettyOut::Ast) = args.unpretty_out {
        salsa::attach(&db, || {
            println!("{:#?}", parsed_file.ast(&db));
        });
    }
}
