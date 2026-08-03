//@ run
//@ exit: 20

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn main() {
    let mut arr: *usize = alloc_array(5);
    let mut i: usize = 0;
    while i < 5 {
        arr[i] = i * 2;
        i = i + 1;
    }

    let mut sum: usize = 0;
    i = 0;
    while i < 5 {
        sum = sum + arr[i];
        i = i + 1;
    }
    __scrap_exit(sum);
}
