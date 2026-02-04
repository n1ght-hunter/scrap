//! Expression lowering context
//!
//! This module defines the ExprLowerer struct which maintains state during
//! expression lowering from AST to IR.

use std::collections::HashMap;

use scrap_ir as ir;
use scrap_shared::ident::Symbol;

use crate::cfg_builder::BasicBlockBuilder;

/// Context for lowering expressions to IR
///
/// This struct maintains the state needed during expression lowering:
/// - Local variable declarations (parameters, named locals, temporaries)
/// - Symbol table for resolving variable names to local IDs
/// - CFG builder for managing basic blocks and control flow
/// - Source text for extracting literal values from spans
pub struct ExprLowerer<'db> {
    pub(crate) db: &'db dyn scrap_shared::Db,
    /// Source text for extracting literal values
    pub(crate) source: &'db str,
    /// All local variable declarations (params + named locals + temps)
    pub local_decls: Vec<ir::LocalDecl<'db>>,
    /// CFG builder for managing basic blocks
    pub cfg_builder: BasicBlockBuilder<'db>,
    /// Symbol table mapping names to local IDs
    pub(crate) symbol_table: HashMap<Symbol<'db>, ir::LocalId>,
}

impl<'db> ExprLowerer<'db> {
    /// Create a new expression lowerer with source text for literal extraction
    pub fn new(db: &'db dyn scrap_shared::Db, source: &'db str) -> Self {
        Self {
            db,
            source,
            local_decls: Vec::new(),
            cfg_builder: BasicBlockBuilder::new(db),
            symbol_table: HashMap::new(),
        }
    }

    /// Allocate an anonymous temporary variable
    pub fn allocate_temp(&mut self, ty: ir::Ty<'db>) -> ir::LocalId {
        let id = ir::LocalId(self.local_decls.len());
        let local_decl = ir::LocalDecl::new(self.db, None, ty);
        self.local_decls.push(local_decl);
        id
    }

    /// Allocate a named local variable
    pub fn allocate_named_local(&mut self, name: Symbol<'db>, ty: ir::Ty<'db>) -> ir::LocalId {
        let id = ir::LocalId(self.local_decls.len());
        let local_decl = ir::LocalDecl::new(self.db, Some(name), ty);
        self.local_decls.push(local_decl);
        id
    }

    /// Emit an assignment statement
    pub fn emit_assign(&mut self, place: ir::Place<'db>, rvalue: ir::Rvalue<'db>) {
        let stmt_kind = ir::StatementKind::Assign(place, rvalue);
        let statement = ir::Statement::new(self.db, stmt_kind);
        self.cfg_builder.emit_statement(statement);
    }

    /// Insert a variable binding into the symbol table
    pub fn insert_binding(&mut self, name: Symbol<'db>, local: ir::LocalId) {
        self.symbol_table.insert(name, local);
    }

    /// Look up a variable binding in the symbol table
    pub fn lookup_binding(&self, name: Symbol<'db>) -> Option<ir::LocalId> {
        self.symbol_table.get(&name).copied()
    }
}
