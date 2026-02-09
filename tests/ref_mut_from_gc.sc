extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let mut x: *usize = box(10);
    let r: &mut usize = &mut x;
    *r = 42;
    ExitProcess(*x);
}
