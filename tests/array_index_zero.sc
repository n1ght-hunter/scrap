//@ run
//@ exit: 101

fn main() {
    let arr: *usize = alloc_array(0);
    let x = arr[0];
    let _ = x;
}
