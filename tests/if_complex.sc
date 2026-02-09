extern "C" {
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let x = 15;
    let y = 3;
    let z = 7;

    let a = if x > 10 && y < 5 {
        1
    } else {
        0
    };

    let b = if x >= 15 && z <= 7 {
        1
    } else {
        0
    };

    let c = if x != 0 && y != 0 {
        1
    } else {
        0
    };

    let d = if x > 100 || z == 7 {
        1
    } else {
        0
    };

    let e = if x < 2 || y > 50 {
        0
    } else {
        1
    };

    let f = if x > 10 && y < 5 && z == 7 {
        1
    } else {
        0
    };

    let result = if a == 1 && b == 1 && c == 1 && d == 1 && e == 1 && f == 1 {
        42
    } else {
        0
    };

    ExitProcess(result);
}
