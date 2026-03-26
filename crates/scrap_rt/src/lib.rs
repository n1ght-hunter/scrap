//! Scrap Runtime
//!
//! Runtime support for the Scrap programming language.
//! Compiled as a staticlib (.lib) and linked into Scrap executables.

#![allow(unsafe_code)]

use scrap_allocator as _;

#[macro_use]
mod sync;
mod coroutine;
mod gc;
