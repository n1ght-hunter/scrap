//! Type table for storing resolved type information.
//!
//! The type table maps AST node IDs to their resolved types,
//! allowing downstream passes (like IR lowering) to look up
//! the type of any expression or local variable.

use scrap_shared::NodeId;

use crate::resolved::ResolvedTy;

/// Type information collected during type checking.
/// Maps AST node IDs to their resolved types.
#[salsa::tracked(debug, persist)]
pub struct TypeTable<'db> {
    /// Expression types as (NodeId, ResolvedTy) pairs
    #[tracked]
    #[returns(ref)]
    pub expr_types: Vec<(NodeId, ResolvedTy<'db>)>,

    /// Local variable types as (NodeId, ResolvedTy) pairs
    #[tracked]
    #[returns(ref)]
    pub local_types: Vec<(NodeId, ResolvedTy<'db>)>,
}

impl<'db> TypeTable<'db> {
    /// Get the type of an expression by its NodeId.
    pub fn expr_type(self, db: &'db dyn scrap_shared::Db, id: NodeId) -> Option<&'db ResolvedTy<'db>> {
        self.expr_types(db)
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, ty)| ty)
    }

    /// Get the type of a local variable by its NodeId.
    pub fn local_type(self, db: &'db dyn scrap_shared::Db, id: NodeId) -> Option<&'db ResolvedTy<'db>> {
        self.local_types(db)
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, ty)| ty)
    }

    /// Check if the table is empty.
    pub fn is_empty(self, db: &'db dyn scrap_shared::Db) -> bool {
        self.expr_types(db).is_empty() && self.local_types(db).is_empty()
    }

    /// Get the number of recorded expression types.
    pub fn expr_count(self, db: &'db dyn scrap_shared::Db) -> usize {
        self.expr_types(db).len()
    }

    /// Get the number of recorded local types.
    pub fn local_count(self, db: &'db dyn scrap_shared::Db) -> usize {
        self.local_types(db).len()
    }
}
