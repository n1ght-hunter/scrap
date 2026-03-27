extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let mut i: usize = 0;
    while i < 42 {
        i = i + 1;
    }
    ExitProcess(i);
}
