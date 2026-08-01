//@ run
//@ exit: 42
//@ manifest: Scrap.toml

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::Counter;

fn main() {
    let c = Counter { n: 42, step: 1 };
    __scrap_exit(c.n);
}
