//@ run
//@ exit: 0
//@ stdout: main start
//@ stdout: sum ok

extern "C" {
    fn __scrap_print(msg: String, len: usize);
}

fn print(msg: String, len: usize) {
    __scrap_print(msg, len);
}

fn worker_a() {
    print("A1\n", 3);
    print("A2\n", 3);
    print("A3\n", 3);
}

fn worker_b() {
    print("B1\n", 3);
    print("B2\n", 3);
}

fn add(a: usize, b: usize) -> usize {
    a + b
}

fn worker_sum(a: usize, b: usize) {
    let result: usize = add(a, b);
    if result == 42 {
        print("sum ok\n", 7);
    }
}

fn main() {
    print("main start\n", 11);

    spawn worker_a();
    spawn worker_b();

    spawn {
        print("block 1\n", 8);
    };

    spawn {
        print("block 2\n", 8);
    };

    spawn worker_sum(35, 7);

    print("main end\n", 9);
}
