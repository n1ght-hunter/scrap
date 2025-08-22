//! Core Abstract Syntax Tree (AST) types for the Scrap language.
//!
//! This module defines the fundamental types used throughout the AST,
//! following the structure and patterns of the official Rust AST.
//! These types serve as the foundation for all other AST nodes.

/// A symbol is an interned string. Symbols are cheap to compare and copy.
/// In a full implementation, this would be backed by an interner for performance.
#[derive(Debug, Clone, Copy)]
pub struct Symbol(pub u32);

/// A unique identifier for AST nodes. NodeIds are used throughout the compiler
/// to track and reference specific nodes during analysis and compilation.
/// Every AST node that can be referenced has a unique NodeId.
#[derive(Debug, Clone, Copy)]
pub struct NodeId(pub u32);

impl NodeId {
    /// Create a new NodeId. In a production compiler, this would be generated
    /// by a proper ID allocation system to ensure uniqueness.
    pub fn new() -> Self {
        NodeId(0)
    }
}

// #[derive(Debug, Clone)]
// pub enum Literal {
//     Int(&str),
//     Float(&str),
//     Str(&str),
// }
