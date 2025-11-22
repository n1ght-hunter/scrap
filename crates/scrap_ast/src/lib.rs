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
pub mod item;
pub mod lit;
pub mod local;
pub mod module;
pub mod node_id;
pub mod operators;
pub mod pat;
pub mod stmt;
pub mod structdef;
pub mod typedef;

pub use shared::*;
mod shared;

pub use node_id::NodeId;

use item::{Item, ItemKind};
use thin_vec::ThinVec;

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Can<'db> {
    pub id: NodeId,
    pub items: ThinVec<Box<Item<'db>>>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Can<'db> {
    fn pretty_print(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        for item in &self.items {
            item.pretty_print(f)?;
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl<'db> Can<'db> {
    pub fn iter_modules(
        &'db self,
    ) -> impl Iterator<Item = (&'db scrap_shared::path::Path<'db>, &'db module::Module<'db>)> {
        self.items.iter().filter_map(|item| {
            if let ItemKind::Module(path, module) = &item.kind {
                Some((&*path, module))
            } else {
                None
            }
        })
    }

    pub fn iter_modules_mut<F>(&'db self, f: F) -> Can<'db>
    where
        F: FnOnce(&mut dyn Iterator<Item = (&scrap_shared::path::Path, &mut module::Module<'db>)>),
    {
        let mut this = self.clone();
        let mut iter = this.items.iter_mut().filter_map(|item| {
            if let ItemKind::Module(path, module) = &mut item.kind {
                Some((&*path, module))
            } else {
                None
            }
        });
        f(&mut iter);
        this
    }
}
