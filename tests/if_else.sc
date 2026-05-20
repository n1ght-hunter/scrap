extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let x = 0;
    let result = if x == 1 {
        10
    } else {
        42
    };
    __scrap_exit(result);
}
