//@ compile-fail
//@ error: cannot borrow `x` as mutable

fn main() {
    let x: usize = 5;
    let r: &mut usize = &mut x;
}
