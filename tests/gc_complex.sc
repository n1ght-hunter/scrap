extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn make_box(val: usize) -> *usize {
    box(val)
}

fn read_ref(r: &usize) -> usize {
    *r
}

fn write_ref(r: &mut usize, val: usize) {
    *r = val;
}

fn add_boxes(a: *usize, b: *usize) -> usize {
    let ra: &usize = &a;
    let rb: &usize = &b;
    let x: usize = *ra;
    let y: usize = *rb;
    x + y
}

fn make_and_add(x: usize, y: usize) -> usize {
    let a: *usize = make_box(x);
    let b: *usize = make_box(y);
    add_boxes(a, b)
}

fn main() {
    let a: *usize = make_box(10);
    let mut b: *usize = make_box(20);
    let c: *usize = make_box(5);
    let mut d: *usize = make_box(1);

    // &mut write through GC pointer: b was 20, now 15
    let rm: &mut usize = &mut b;
    *rm = 15;

    // write via function taking &mut: d was 1, now 7
    write_ref(&mut d, 7);

    // read d via &mut (no conflicting & borrow)
    let val_d: usize = *d;

    let sum1: usize = add_boxes(a, b);
    let val_c: usize = read_ref(&c);
    let sum2: usize = make_and_add(val_c, val_d);

    // &mut on a fresh box: write then read back
    let mut e: *usize = make_box(0);
    let re: &mut usize = &mut e;
    *re = 6;
    let doubled: usize = *e;

    ExitProcess(sum1 + sum2 + doubled);
}
