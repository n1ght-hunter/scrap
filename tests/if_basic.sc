//@ run
//@ exit: 42

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let x = 1;
    let mut result = 0;
    if x == 1 {
        result = 42;
    }
    __scrap_exit(result);
}
