//@ compile-fail
//@ error: has no field named `x`

struct Point {
}
struct Point2 {
    x: usize,
    y: usize,
}

fn main() {
    let p = Point { x: 42, y: 10 };
    let t = Point2 {  };
}
