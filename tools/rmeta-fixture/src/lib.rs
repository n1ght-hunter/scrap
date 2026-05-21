//! A tiny fixture crate with a known public API, used to verify the scrap-rustc
//! metadata driver extracts correct symbols, ABI, and layouts cross-crate.

pub struct Point {
    pub x: i32,
    pub y: i64,
}

#[repr(C)]
pub struct CPoint {
    pub x: i32,
    pub y: i64,
}

pub enum Shape {
    Unit,
    Circle(f64),
    Rect { w: f64, h: f64 },
}

pub fn scalar_add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn add_usize(a: usize, b: usize) -> usize {
    a + b
}

pub fn make_point(x: i32, y: i64) -> Point {
    Point { x, y }
}

pub fn identity<T>(x: T) -> T {
    x
}

/// All-`pub` scalar struct — constructible field-by-field from Scrap.
pub struct Counter {
    pub n: usize,
    pub step: usize,
}

/// Has a private field — Scrap may read `visible` but not construct it.
pub struct Secret {
    pub visible: usize,
    hidden: usize,
}

impl Secret {
    pub fn hidden(&self) -> usize {
        self.hidden
    }
}
