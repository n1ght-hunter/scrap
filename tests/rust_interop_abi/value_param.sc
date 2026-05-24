extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::Pair;
use rust::rmeta_fixture::make_pair;
use rust::rmeta_fixture::pair_sum;

// A by-value native Rust value in a *user* Scrap function's signature has no
// Scrap-side ABI lowering yet — codegen must reject this cleanly (pass `&Pair`),
// not silently miscompile.
fn id_pair(p: Pair) -> Pair {
    p
}

fn main() {
    let p = make_pair(40, 2);
    let q = id_pair(p);
    __scrap_exit(pair_sum(q));
}
