//@ run
//@ exit: 50

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
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
    __scrap_exit(sum);
}
