//@ compile-fail
//@ error: cannot assign to immutable variable

fn main() {
    let x: usize = 5;
    x = 42;
}
