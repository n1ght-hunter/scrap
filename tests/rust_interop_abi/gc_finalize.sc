//@ run
//@ exit: 1
//@ manifest: Scrap.toml

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
    fn __scrap_gc_collect();
}

use rust::rmeta_fixture::Dropper;
use rust::rmeta_fixture::make_dropper;
use rust::rmeta_fixture::dropped_count;

fn stash() {
    let d = make_dropper(7);
    let b = box(d);
}

fn main() {
    stash();
    __scrap_gc_collect();
    __scrap_exit(dropped_count());
}
