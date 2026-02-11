extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
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
    ExitProcess(result);
}

fn main() {
    spawn worker(500);
}
