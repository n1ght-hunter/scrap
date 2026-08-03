//@ compile-fail

fn main() {
    let arr: *usize = alloc_array(2);
    arr[0] = 1;
}
