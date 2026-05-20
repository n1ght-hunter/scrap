extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn deep(n: usize) -> usize {
    if n <= 0 {
        return 42;
    }
    let result = deep(n - 1);
    result
}

fn worker(n: usize) {
    let result = deep(n);
    __scrap_exit(result);
}

fn main() {
    spawn worker(500);
}
