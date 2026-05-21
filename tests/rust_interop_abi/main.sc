extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::make_pair;
use rust::rmeta_fixture::pair_sum;

fn main() {
    let p = make_pair(40, 2);
    __scrap_exit(pair_sum(p));
}
