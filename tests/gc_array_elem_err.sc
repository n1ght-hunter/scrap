//@ compile-fail
//@ error: does not support element type

struct Point {
    x: usize,
    y: usize,
}

fn main() {
    let arr: *Point = alloc_array(3);
}
