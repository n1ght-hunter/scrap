extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::id_big;

fn main() {
    let x = Big { a: 1, b: 2, c: 3, d: 4 };
    let y = id_big(x);
    __scrap_exit(y.a);
}
