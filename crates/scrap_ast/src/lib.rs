//! Core Abstract Syntax Tree (AST) types for the Scrap language.
//!
//! This module defines the fundamental types used throughout the AST,
//! following the structure and patterns of the official Rust AST.
//! These types serve as the foundation for all other AST nodes.
//! 


pub mod node_id;
pub mod typedef;
pub mod ident;
pub mod item;
pub mod lit;
pub mod path;
pub mod fndef;
pub mod pat;
pub mod block;
pub mod expr;
pub mod local;
pub mod stmt;
pub mod enumdef;
pub mod field;
pub mod operators;
pub mod structdef;


mod shared;
pub use shared::*;
