extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let mut x: usize = 5;
    let r: &usize = &x;
    *r = 10;
    ExitProcess(*r);
}
