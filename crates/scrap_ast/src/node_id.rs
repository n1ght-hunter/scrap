/// A unique identifier for AST nodes. NodeIds are used throughout the compiler
/// to track and reference specific nodes during analysis and compilation.
/// Every AST node that can be referenced has a unique NodeId.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId {
    id: u32,
    file_hash: u64,
}

impl NodeId {
    /// Get the raw ID value (useful for debugging and serialization)
    pub fn as_u32(self) -> u32 {
        self.id
    }

    /// Create a NodeId from a raw u32 value (should only be used for deserialization)
    pub fn new(id: u32, file_hash: u64) -> Self {
        NodeId { id, file_hash }
    }

    /// Create a special invalid NodeId for error recovery cases
    pub fn invalid() -> Self {
        NodeId {
            id: u32::MAX,
            file_hash: 0,
        }
    }
}
