//! Control Flow Graph (CFG) builder for IR
//!
//! This module handles the construction of basic blocks and control flow
//! for function bodies. It manages block allocation, statement accumulation,
//! and terminator placement.

use scrap_ir as ir;

/// Builder for constructing a Control Flow Graph (CFG)
///
/// The CFG is represented as a collection of basic blocks, where each block
/// contains a sequence of statements and ends with a terminator that controls
/// the flow to other blocks.
pub struct BasicBlockBuilder<'db> {
    db: &'db dyn scrap_shared::Db,
    /// All basic blocks in the CFG
    blocks: Vec<BlockData<'db>>,
    /// The ID of the current block being built
    current_block_id: ir::BasicBlockId,
}

/// Internal representation of a basic block during construction
struct BlockData<'db> {
    /// Statements in this block
    statements: Vec<ir::Statement<'db>>,
    /// Terminator for this block (None if not yet finalized)
    terminator: Option<ir::Terminator<'db>>,
}

impl<'db> BasicBlockBuilder<'db> {
    /// Create a new CFG builder with an initial empty block
    pub fn new(db: &'db dyn scrap_shared::Db) -> Self {
        let initial_block = BlockData {
            statements: Vec::new(),
            terminator: None,
        };

        Self {
            db,
            blocks: vec![initial_block],
            current_block_id: ir::BasicBlockId(0),
        }
    }

    /// Get the ID of the current block
    pub fn current_block(&self) -> ir::BasicBlockId {
        self.current_block_id
    }

    /// Start a new basic block and return its ID
    ///
    /// This allocates a new block but doesn't switch to it automatically.
    /// Use `set_current_block` to switch to the new block.
    pub fn start_block(&mut self) -> ir::BasicBlockId {
        let block_id = ir::BasicBlockId(self.blocks.len());
        let block = BlockData {
            statements: Vec::new(),
            terminator: None,
        };
        self.blocks.push(block);
        block_id
    }

    /// Switch to a different block for subsequent operations
    pub fn set_current_block(&mut self, block_id: ir::BasicBlockId) {
        self.current_block_id = block_id;
    }

    /// Add a statement to the current block
    pub fn emit_statement(&mut self, statement: ir::Statement<'db>) {
        let block = &mut self.blocks[self.current_block_id.0];

        // Can't add statements after a terminator
        if block.terminator.is_some() {
            // This is dead code - create a new unreachable block
            let unreachable_id = self.start_block();
            self.set_current_block(unreachable_id);
            self.blocks[unreachable_id.0].statements.push(statement);
            return;
        }

        block.statements.push(statement);
    }

    /// Finish the current block with a terminator
    ///
    /// After calling this, the current block is complete and no more
    /// statements can be added to it.
    pub fn finish_block(&mut self, terminator: ir::Terminator<'db>) {
        let block = &mut self.blocks[self.current_block_id.0];

        // If block already has a terminator, this is dead code
        if block.terminator.is_some() {
            return;
        }

        block.terminator = Some(terminator);
    }

    /// Build the final CFG, consuming the builder
    ///
    /// This converts all block data into IR BasicBlock instances.
    /// Any blocks without terminators will get an Unreachable terminator.
    pub fn build(self) -> Vec<ir::BasicBlock<'db>> {
        self.blocks
            .into_iter()
            .map(|block_data| {
                let terminator = block_data.terminator.unwrap_or(ir::Terminator::Unreachable);
                ir::BasicBlock::new(self.db, block_data.statements, terminator)
            })
            .collect()
    }

    /// Get the number of blocks currently allocated
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Check if the current block is terminated
    pub fn current_block_is_terminated(&self) -> bool {
        self.blocks[self.current_block_id.0].terminator.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scrap_shared::types::IntVal;

    #[scrap_macros::salsa_test]
    fn test_new_builder(db: &dyn scrap_shared::Db) {
        let builder = BasicBlockBuilder::new(db);

        // Should start with one empty block
        assert_eq!(builder.block_count(), 1);
        assert_eq!(builder.current_block(), ir::BasicBlockId(0));
    }

    #[scrap_macros::salsa_test]
    fn test_start_block(db: &dyn scrap_shared::Db) {
        let mut builder = BasicBlockBuilder::new(db);

        let bb1 = builder.start_block();
        assert_eq!(bb1, ir::BasicBlockId(1));
        assert_eq!(builder.block_count(), 2);

        let bb2 = builder.start_block();
        assert_eq!(bb2, ir::BasicBlockId(2));
        assert_eq!(builder.block_count(), 3);
    }

    #[scrap_macros::salsa_test]
    fn test_set_current_block(db: &dyn scrap_shared::Db) {
        let mut builder = BasicBlockBuilder::new(db);

        let bb1 = builder.start_block();
        builder.set_current_block(bb1);

        assert_eq!(builder.current_block(), ir::BasicBlockId(1));
    }

    #[scrap_macros::salsa_test]
    fn test_emit_statement(db: &dyn scrap_shared::Db) {
        let mut builder = BasicBlockBuilder::new(db);

        // Create a simple assignment statement
        let place = ir::Place::Local(ir::LocalId(0));
        let constant = ir::Constant::Int(IntVal::I32(42));
        let rvalue = ir::Rvalue::Constant(constant);
        let stmt_kind = ir::StatementKind::Assign(place, rvalue);
        let statement = ir::Statement::new(db, stmt_kind);

        builder.emit_statement(statement);

        // Build and check
        let blocks = builder.build();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].statements(db).len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_finish_block(db: &dyn scrap_shared::Db) {
        let mut builder = BasicBlockBuilder::new(db);

        builder.finish_block(ir::Terminator::Return);

        let blocks = builder.build();
        assert_eq!(blocks[0].terminator(db), ir::Terminator::Return);
    }

    #[scrap_macros::salsa_test]
    fn test_build_simple_cfg(db: &dyn scrap_shared::Db) {
        let mut builder = BasicBlockBuilder::new(db);

        // BB0: goto BB1
        let bb1 = builder.start_block();
        builder.finish_block(ir::Terminator::Goto { target: bb1 });

        // BB1: return
        builder.set_current_block(bb1);
        builder.finish_block(ir::Terminator::Return);

        let blocks = builder.build();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0].terminator(db), ir::Terminator::Goto { .. }));
        assert_eq!(blocks[1].terminator(db), ir::Terminator::Return);
    }

    #[scrap_macros::salsa_test]
    fn test_dead_code_after_terminator(db: &dyn scrap_shared::Db) {
        let mut builder = BasicBlockBuilder::new(db);

        // Add a return terminator
        builder.finish_block(ir::Terminator::Return);

        // Try to add a statement after terminator (dead code)
        let place = ir::Place::Local(ir::LocalId(0));
        let constant = ir::Constant::Int(IntVal::I32(42));
        let rvalue = ir::Rvalue::Constant(constant);
        let stmt_kind = ir::StatementKind::Assign(place, rvalue);
        let statement = ir::Statement::new(db, stmt_kind);

        builder.emit_statement(statement);

        // Should have created a new unreachable block
        let blocks = builder.build();
        assert!(blocks.len() >= 2); // At least the original block and the unreachable one
    }

    #[scrap_macros::salsa_test]
    fn test_current_block_is_terminated(db: &dyn scrap_shared::Db) {
        let mut builder = BasicBlockBuilder::new(db);

        assert!(!builder.current_block_is_terminated());

        builder.finish_block(ir::Terminator::Return);

        assert!(builder.current_block_is_terminated());
    }
}
