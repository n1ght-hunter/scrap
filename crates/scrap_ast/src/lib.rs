//! Core Abstract Syntax Tree (AST) types for the Scrap language.
//!
//! This module defines the fundamental types used throughout the AST,
//! following the structure and patterns of the official Rust AST.
//! These types serve as the foundation for all other AST nodes.
//!

pub mod block;
pub mod enumdef;
pub mod expr;
pub mod field;
pub mod fndef;
pub mod ident;
pub mod item;
pub mod lit;
pub mod local;
pub mod module;
pub mod node_id;
pub mod operators;
pub mod pat;
pub mod path;
pub mod stmt;
pub mod structdef;
pub mod typedef;

pub use shared::*;
mod shared;

use item::Item;
use node_id::NodeId;
use thin_vec::ThinVec;

#[derive(Clone, Debug)]
pub struct Can {
    pub id: NodeId,
    pub items: ThinVec<Box<Item>>,
}
