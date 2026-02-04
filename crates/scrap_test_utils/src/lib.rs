//! Test utilities for the Scrap compiler project.
//!
//! This crate provides common testing utilities used across multiple test suites.

use std::sync::LazyLock;

// Re-export commonly used testing tools
pub use insta;
pub use scrap_macros::salsa_test;

/// Remove salsa IDs from debug output to make snapshots stable.
///
/// Salsa assigns non-deterministic IDs to tracked structs, which makes
/// snapshot testing difficult. This function strips out all `[salsa id]: Id(N),`
/// lines from the debug output, including the entire line if it only contains
/// the salsa ID and whitespace.
pub fn strip_salsa_ids(s: &str) -> String {
    static RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"[ \t]*\[salsa id\]: Id\([\da-fA-F]+\),[ \t]*\n").unwrap());
    RE.replace_all(s, "").to_string()
}

#[macro_export]
/// A helper macro to assert snapshots while stripping salsa IDs.
/// This macro takes an optional name and an expression, formats it with debug formatting,
/// strips salsa IDs, and then asserts the snapshot using `insta`.
macro_rules! salsa_assert_snapshot {
    ($name:expr, $arg:expr) => {
        $crate::insta::assert_snapshot!($name, $crate::strip_salsa_ids(&format!("{:#?}", $arg)));
    };
    ($arg:expr) => {
        $crate::insta::assert_snapshot!($crate::strip_salsa_ids(&format!("{:#?}", $arg)));
    };
}
