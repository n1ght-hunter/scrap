//! Type table for storing resolved type information.
//!
//! The type table maps AST node IDs to their resolved types,
//! allowing downstream passes (like IR lowering) to look up
//! the type of any expression or local variable.

use scrap_shared::NodeId;
use scrap_shared::ident::Symbol;

use crate::resolved::ResolvedTy;

/// Type information collected during type checking.
/// Maps AST node IDs to their resolved types.
#[salsa::tracked(debug, persist)]
pub struct TypeTable<'db> {
    /// Expression types as (NodeId, ResolvedTy) pairs
    #[tracked]
    #[returns(ref)]
    pub expr_types: Vec<(NodeId, ResolvedTy)>,

    /// Local variable types as (NodeId, ResolvedTy) pairs
    #[tracked]
    #[returns(ref)]
    pub local_types: Vec<(NodeId, ResolvedTy)>,

    /// Inferred function return types as (Symbol, ResolvedTy) pairs.
    /// Only populated when the inferred return type differs from the declared one
    /// (e.g., a function with no return annotation whose body diverges).
    #[tracked]
    #[returns(ref)]
    pub fn_return_types: Vec<(Symbol, ResolvedTy)>,

    /// Generic function instantiations: (fn_name, call_node_id, [(type_param, concrete_type)])
    #[tracked]
    #[returns(ref)]
    pub generic_instantiations: Vec<(Symbol, NodeId, Vec<(Symbol, ResolvedTy)>)>,
}

impl<'db> TypeTable<'db> {
    /// Get the type of an expression by its NodeId.
    pub fn expr_type(self, db: &'db dyn scrap_shared::Db, id: NodeId) -> Option<&'db ResolvedTy> {
        self.expr_types(db)
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, ty)| ty)
    }

    /// Get the type of a local variable by its NodeId.
    pub fn local_type(self, db: &'db dyn scrap_shared::Db, id: NodeId) -> Option<&'db ResolvedTy> {
        self.local_types(db)
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, ty)| ty)
    }

    /// Get the inferred return type of a function by its name.
    pub fn fn_return_type(
        self,
        db: &'db dyn scrap_shared::Db,
        name: Symbol,
    ) -> Option<&'db ResolvedTy> {
        self.fn_return_types(db)
            .iter()
            .find(|(sym, _)| *sym == name)
            .map(|(_, ty)| ty)
    }

    /// Get the generic instantiation for a call site.
    pub fn generic_instantiation(
        self,
        db: &'db dyn scrap_shared::Db,
        call_site: NodeId,
    ) -> Option<&'db (Symbol, NodeId, Vec<(Symbol, ResolvedTy)>)> {
        self.generic_instantiations(db)
            .iter()
            .find(|(_, node_id, _)| *node_id == call_site)
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
