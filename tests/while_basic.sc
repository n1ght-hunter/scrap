extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let mut i: usize = 0;
    while i < 42 {
        i = i + 1;
    }
    __scrap_exit(i);
}
