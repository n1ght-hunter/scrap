//@ compile-fail
//@ error: cannot construct `Secret`
//@ manifest: Scrap.toml

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

use rust::rmeta_fixture::Secret;

fn main() {
    let s = Secret { visible: 1, hidden: 2 };
    __scrap_exit(s.visible);
}
