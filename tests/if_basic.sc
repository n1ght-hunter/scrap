extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let x = 1;
    let result = 0;
    if x == 1 {
        result = 42;
    }
    ExitProcess(result);
}
