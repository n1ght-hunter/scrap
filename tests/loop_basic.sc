//@ run
//@ exit: 42

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let mut i: usize = 0;
    loop {
        if i == 42 {
            break;
        }
        i = i + 1;
    }
    __scrap_exit(i);
}
