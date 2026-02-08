extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

struct Point {
    x: usize,
    y: usize,
}

fn main() {
    let p = Point { x: 42, y: 10 };
    ExitProcess(p.x);
}
