extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn worker(code: usize) {
    __scrap_exit(code);
}

fn main() {
    spawn worker(42);
}
