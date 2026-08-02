//@ run
//@ exit: 127

extern "C" {
    fn __scrap_exit(exit_code: usize) -> !;
}

fn nan_from(x: f64) -> f64 {
    x / 0.0
}

fn main() {
    let two = 2.0;
    let three = 3.0;
    let mut score = 0;

    if two < three {
        score = score + 1;
    }
    if three > two {
        score = score + 2;
    }
    if two <= two {
        score = score + 4;
    }
    if two >= two {
        score = score + 8;
    }
    if two == two {
        score = score + 16;
    }
    if two != three {
        score = score + 32;
    }

    // NaN is unordered: every comparison but `!=` must be false.
    let nan = nan_from(0.0);
    if nan != nan {
        score = score + 64;
    }
    if nan < two {
        score = score + 128;
    }
    if nan >= two {
        score = score + 256;
    }
    if nan == nan {
        score = score + 512;
    }

    __scrap_exit(score);
}
