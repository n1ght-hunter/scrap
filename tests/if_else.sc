extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let x = 0;
    let result = if x == 1 {
        10
    } else {
        42
    };
    ExitProcess(result);
}
