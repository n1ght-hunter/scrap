#![feature(try_blocks)]

mod args;

use std::ffi::OsString;

use args::UnPrettyOut;
use clap::Parser;
use scrap_errors::SimpleError;

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

    if args.verbose {
        scrap_errors::set_verbose_errors(true);
    }

    let mut db = scrap_shared::salsa::ScrapDb::default();

    let load_config = bincode::config::standard();

    if let Some(db_snapshot_path) = args.db_snapshot.as_ref() {
        if db_snapshot_path.exists() {
            let serialized = std::fs::read_to_string(db_snapshot_path.with_extension("json"))
                .sexpect("Failed to read JSON database snapshot for debugging purposes");
            <dyn salsa::Database>::deserialize(
                &mut db,
                &mut serde_json::Deserializer::from_str(&serialized),
            )
            .sunwrap();

            let mut file = std::io::BufReader::new(
                std::fs::File::open(db_snapshot_path).sexpect("Failed to open database snapshot"),
            );
            <dyn salsa::Database>::deserialize(
                &mut db,
                bincode::serde::OwnedSerdeDecoder::from_reader(&mut file, load_config)
                    .as_deserializer(),
            )
            .sunwrap();
        } else {
            tracing::info!(
                "Database snapshot file '{}' does not exist. Starting with a fresh database.",
                db_snapshot_path.display()
            );
        }
    }

    || -> anyhow::Result<()> {
        let input_path = scrap_shared::salsa::InputPath::new(&db, args.source_file.clone());
        let input_file = scrap_shared::salsa::load_file(&db, input_path);
        let lexed_tokens = scrap_lexer::lex_file(&db, input_file);
        let parsed_file = scrap_parser::parse_tokens(&db, input_file, lexed_tokens, true);

        if let Some(UnPrettyOut::Ast) = args.unpretty_out {
            salsa::attach(&db, || {
                println!("{:#?}", parsed_file.ast(&db));
            });
            return Ok(());
        }

        Ok(())
    }()
    .sunwrap();

    if let Some(db_snapshot_path) = args.db_snapshot.as_ref() {
        std::fs::create_dir_all(
            db_snapshot_path
                .parent()
                .expect("Failed to get parent directory of database snapshot path"),
        )
        .sexpect("Failed to create parent directory for database snapshot");

        let mut file =
            std::fs::File::create(db_snapshot_path).expect("Failed to create database snapshot");
        bincode::serde::encode_into_std_write(
            <dyn salsa::Database>::as_serialize(&mut db),
            &mut file,
            load_config,
        )
        .sunwrap();

        let output =
            serde_json::to_string_pretty(&<dyn salsa::Database>::as_serialize(&mut db)).unwrap();
        std::fs::write(db_snapshot_path.with_extension("json"), output)
            .sexpect("Failed to write JSON database snapshot");
    }
}
