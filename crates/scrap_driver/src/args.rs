#[derive(Debug, Clone, PartialEq, Eq, Hash, Parser)]
pub struct Args {
    /// The type of output to generate
    #[clap(long = "unpretty-out")]
    pub unpretty_out: Option<UnPrettyOut>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum)]
pub enum UnPrettyOut {
    Ast,
    Mir,
    CLIR,
}