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

/// A 16-byte all-`pub` struct — passed/returned by value as a `ScalarPair`.
pub struct Pair {
    pub a: usize,
    pub b: usize,
}

/// Returns a `Pair` by value (ScalarPair return).
pub fn make_pair(a: usize, b: usize) -> Pair {
    Pair { a, b }
}

/// Takes a `Pair` by value (ScalarPair argument), returns a scalar.
pub fn pair_sum(p: Pair) -> usize {
    p.a + p.b
}

/// A 32-byte struct — passed/returned by value via `Indirect` (sret), which is
/// not yet ABI-lowered (used to exercise the "unsupported mode" diagnostic).
pub struct Big {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub d: usize,
}

pub fn id_big(x: Big) -> Big {
    x
}

/// Holds a non-scalar field (`inner: Pair`) plus a scalar — importable as an
/// opaque-field type, passed/returned by value.
pub struct Wrapper {
    pub inner: Pair,
    pub tag: usize,
}

pub fn wrap(p: Pair, tag: usize) -> Wrapper {
    Wrapper { inner: p, tag }
}

pub fn wrapper_tag(w: Wrapper) -> usize {
    w.tag
}

use std::sync::atomic::{AtomicUsize, Ordering};

static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

/// A type whose `Drop` bumps a process-global counter, so Scrap RAII drops are
/// observable via `dropped_count()`.
pub struct Dropper {
    pub tag: usize,
}

impl Drop for Dropper {
    fn drop(&mut self) {
        DROP_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn make_dropper(tag: usize) -> Dropper {
    Dropper { tag }
}

/// Consumes (and thus drops) a `Dropper` — used to verify a moved value is not
/// also dropped by the caller.
pub fn take_dropper(_d: Dropper) {}

pub fn dropped_count() -> usize {
    DROP_COUNT.load(Ordering::SeqCst)
}
