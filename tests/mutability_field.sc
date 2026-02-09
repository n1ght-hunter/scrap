extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

struct Point {
    x: usize,
    y: usize,
}

fn main() {
    let mut p = Point { x: 10, y: 20 };
    p.x = 42;
    ExitProcess(p.x);
}
