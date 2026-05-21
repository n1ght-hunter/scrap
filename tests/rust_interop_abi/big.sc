extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::id_big;

fn main() {
    let x = Big { a: 42, b: 1, c: 2, d: 3 };
    let y = id_big(x);
    __scrap_exit(y.a);
}
