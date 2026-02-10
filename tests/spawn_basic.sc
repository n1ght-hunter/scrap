extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn worker(code: usize) {
    ExitProcess(code);
}

fn main() {
    spawn worker(42);
}
