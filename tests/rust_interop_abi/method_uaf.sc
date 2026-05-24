extern "C" { fn __scrap_exit(exit_code: usize) -> !; }
use rust::rmeta_fixture::make_dropper;
use rust::rmeta_fixture::Dropper;
fn main() {
    let d = make_dropper(9);
    let a = d.consume();
    let b = d.consume();
    __scrap_exit(a);
}
