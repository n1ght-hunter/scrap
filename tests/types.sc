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
    let x = 5;
    let y: usize = 10;
    let result = add(x, y);
    exit(result);
}
