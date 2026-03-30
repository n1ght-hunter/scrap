//! Type table for storing resolved type information.
//!
//! Plain struct with HashMap fields for O(1) lookups.
//! Returned from `check_types` via salsa `returns(ref)`.

use hashbrown::HashMap;

use scrap_shared::NodeId;
use scrap_shared::ident::Symbol;

use crate::resolved::ResolvedTy;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TypeTable {
    expr_types: HashMap<NodeId, ResolvedTy>,
    local_types: HashMap<NodeId, ResolvedTy>,
    fn_return_types: HashMap<Symbol, ResolvedTy>,
    generic_instantiations: HashMap<Symbol, Vec<(NodeId, Vec<(Symbol, ResolvedTy)>)>>,
}

impl TypeTable {
    pub(crate) fn new(
        expr_types: HashMap<NodeId, ResolvedTy>,
        local_types: HashMap<NodeId, ResolvedTy>,
        fn_return_types: HashMap<Symbol, ResolvedTy>,
        generic_instantiations: HashMap<Symbol, Vec<(NodeId, Vec<(Symbol, ResolvedTy)>)>>,
    ) -> Self {
        Self {
            expr_types,
            local_types,
            fn_return_types,
            generic_instantiations,
        }
    }

    pub fn empty() -> Self {
        Self {
            expr_types: HashMap::new(),
            local_types: HashMap::new(),
            fn_return_types: HashMap::new(),
            generic_instantiations: HashMap::new(),
        }
    }

    pub fn insert_expr_type(&mut self, id: NodeId, ty: ResolvedTy) {
        self.expr_types.insert(id, ty);
    }

    pub fn expr_type(&self, id: NodeId) -> Option<&ResolvedTy> {
        self.expr_types.get(&id)
    }

    pub fn local_type(&self, id: NodeId) -> Option<&ResolvedTy> {
        self.local_types.get(&id)
    }

    pub fn fn_return_type(&self, name: Symbol) -> Option<&ResolvedTy> {
        self.fn_return_types.get(&name)
    }

    pub fn generic_instantiations_for(
        &self,
        name: Symbol,
    ) -> Option<&Vec<(NodeId, Vec<(Symbol, ResolvedTy)>)>> {
        self.generic_instantiations.get(&name)
    }

    pub fn all_generic_instantiations(
        &self,
    ) -> &HashMap<Symbol, Vec<(NodeId, Vec<(Symbol, ResolvedTy)>)>> {
        &self.generic_instantiations
    }

    pub fn is_empty(&self) -> bool {
        self.expr_types.is_empty() && self.local_types.is_empty()
    }
}
