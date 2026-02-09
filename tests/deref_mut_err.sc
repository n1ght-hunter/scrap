extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let x: *usize = box(42);
    *x = 99;
    ExitProcess(*x);
}
