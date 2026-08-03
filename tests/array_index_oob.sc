//@ run
//@ exit: 101

fn main() {
    let arr: *usize = alloc_array(2);
    let x = arr[2];
    let _ = x;
}
