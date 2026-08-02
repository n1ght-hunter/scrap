//@ run
//@ exit: 101
//@ stdout: attempt to add with overflow

fn add(a: usize, b: usize) -> usize {
    a + b
}

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn exit(code: usize) {
    __scrap_exit(code);
}

fn main() {
    let max: usize = 18446744073709551615;
    let result = add(max, 1);
    exit(result);
}
