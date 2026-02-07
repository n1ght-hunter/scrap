fn add(a: usize, b: usize) -> usize {
    a + b
}

extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn exit(code: usize) {
    ExitProcess(code);
}

fn main() {
    let max: usize = 18446744073709551615;
    let result = add(max, 1);
    exit(result);
}
