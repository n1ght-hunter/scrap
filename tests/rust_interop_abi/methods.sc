//@ run
//@ exit: 2
//@ manifest: Scrap.toml

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::Tally;

fn main() {
    let mut c = Tally::new();
    c.inc();
    c.inc();
    __scrap_exit(c.get());
}
