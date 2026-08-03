//@ compile-fail

fn main() {
    let arr: *usize = alloc_array(2);
    let r: &usize = &arr[0];
    let x = r[0];
    let _ = x;
}
