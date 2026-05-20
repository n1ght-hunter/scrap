extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let x: *usize = box(42);
    *x = 99;
    __scrap_exit(*x);
}
