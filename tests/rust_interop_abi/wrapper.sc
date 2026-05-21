extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::make_pair;
use rust::rmeta_fixture::wrap;
use rust::rmeta_fixture::wrapper_tag;

fn main() {
    let p = make_pair(1, 2);
    let w = wrap(p, 42);
    __scrap_exit(wrapper_tag(w));
}
