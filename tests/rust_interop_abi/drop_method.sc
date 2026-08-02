//@ run
//@ exit: 1
//@ manifest: Scrap.toml

extern "C" { fn __scrap_exit(exit_code: usize) -> !; }
use rust::rmeta_fixture::make_dropper;
use rust::rmeta_fixture::Dropper;
use rust::rmeta_fixture::dropped_count;
fn use_it() {
    let d = make_dropper(9);
    d.value();
}
fn main() {
    use_it();
    __scrap_exit(dropped_count());
}
