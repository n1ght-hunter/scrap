extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let x = 3;
    let result = if x == 1 {
        10
    } else if x == 2 {
        20
    } else {
        42
    };
    ExitProcess(result);
}
