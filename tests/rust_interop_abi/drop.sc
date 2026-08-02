//@ run
//@ exit: 1
//@ manifest: Scrap.toml

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::make_dropper;
use rust::rmeta_fixture::dropped_count;

fn consume() {
    let d = make_dropper(7);
}

fn main() {
    consume();
    __scrap_exit(dropped_count());
}
