extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    spawn {
        __scrap_exit(42);
    };
}
