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

use scrap_shared::id::ModuleId;
pub use shared::*;
mod shared;

pub use node_id::NodeId;

use item::{Item, ItemKind};
use thin_vec::ThinVec;

#[salsa::tracked(debug, persist)]
pub struct Can<'db> {
    pub id: NodeId,
    #[returns(ref)]
    pub name: ModuleId<'db>,
    #[returns(ref)]
    pub items: ThinVec<Box<Item<'db>>>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Can<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        salsa::with_attached_database(|db| {
            for item in self.items(db) {
                item.pretty_print_indent(f, indent)?;
                writeln!(f)?;
            }
            Ok(())
        })
        .unwrap_or_else(|| f.write_str("<no db>"))?;

        Ok(())
    }
}

impl<'db> Can<'db> {
    pub fn iter_modules(
        &'db self,
        db: &'db dyn scrap_shared::Db,
    ) -> impl Iterator<Item = module::Module<'db>> {
        self.items(db).iter().filter_map(|item| {
            if let ItemKind::Module(module) = item.kind {
                Some(module)
            } else {
                None
            }
        })
    }
}
