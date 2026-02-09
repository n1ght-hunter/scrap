extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let mut x: usize = 5;
    x = 42;
    ExitProcess(x);
}
