extern "C" { fn __scrap_exit(c: usize) -> !; }
use rust::rmeta_fixture::Dropper;
use rust::rmeta_fixture::make_dropper;
fn main() {
    let d = make_dropper(7);
    let b = box(d);
    let x = d.value();
    __scrap_exit(x);
}
