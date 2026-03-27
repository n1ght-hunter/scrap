extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let mut i: usize = 0;
    loop {
        if i == 42 {
            break;
        }
        i = i + 1;
    }
    ExitProcess(i);
}
