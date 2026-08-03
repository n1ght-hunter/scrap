//@ run
//@ exit: 101

fn main() {
    let mut arr: *usize = alloc_array(2);
    arr[2] = 1;
}
