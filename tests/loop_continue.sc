extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let mut i: usize = 0;
    let mut sum: usize = 0;
    while i < 10 {
        i = i + 1;
        if i == 5 {
            continue;
        }
        sum = sum + i;
    }
    ExitProcess(sum);
}
