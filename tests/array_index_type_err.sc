//@ compile-fail
//@ error: usize

fn main() {
    let arr: *usize = alloc_array(2);
    let x = arr[true];
    let _ = x;
}
