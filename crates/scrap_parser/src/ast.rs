//! Core Abstract Syntax Tree (AST) types for the Scrap language.
//!
//! This module defines the fundamental types used throughout the AST,
//! following the structure and patterns of the official Rust AST.
//! These types serve as the foundation for all other AST nodes.

use std::sync::atomic::{AtomicU32, Ordering};

/// Global counter for generating unique NodeIds across the entire AST.
/// This ensures that every AST node gets a unique identifier.
static NODE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// A symbol is an interned string. Symbols are cheap to compare and copy.
/// In a full implementation, this would be backed by an interner for performance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(pub u32);

/// A unique identifier for AST nodes. NodeIds are used throughout the compiler
/// to track and reference specific nodes during analysis and compilation.
/// Every AST node that can be referenced has a unique NodeId.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeId {
    /// Create a new unique NodeId. Each call to this function returns a different ID,
    /// ensuring that every AST node has a unique identifier across the entire program.
    pub fn new() -> Self {
        let id = NODE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        NodeId(id)
    }

    /// Get the raw ID value (useful for debugging and serialization)
    pub fn as_u32(self) -> u32 {
        self.0
    }

    /// Create a NodeId from a raw u32 value (should only be used for deserialization)
    pub fn from_u32(id: u32) -> Self {
        NodeId(id)
    }
}

// #[derive(Debug, Clone)]
// pub enum Literal {
//     Int(&str),
//     Float(&str),
//     Str(&str),
// }
