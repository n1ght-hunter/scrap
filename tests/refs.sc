extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let x: usize = 42;
    let r: &usize = &x;
    ExitProcess(*r);
}
