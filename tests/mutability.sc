//@ run
//@ exit: 42

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let mut x: usize = 5;
    x = 42;
    __scrap_exit(x);
}
