extern "C" { fn __scrap_exit(exit_code: usize) -> !; }
use rust::rmeta_fixture::make_dropper;
use rust::rmeta_fixture::take_dropper;
use rust::rmeta_fixture::dropped_count;
fn main() {
    let d = make_dropper(7);
    take_dropper(d);
    __scrap_exit(dropped_count());
}
