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
/// - Type table for looking up types resolved during type checking
/// Info about a single enum variant for lowering.
#[derive(Debug, Clone)]
pub enum VariantInfo<'db> {
    Unit,
    Tuple(Vec<ir::Ty<'db>>),
    Struct(Vec<(Symbol<'db>, ir::Ty<'db>)>),
}

/// Info about all variants of an enum.
#[derive(Debug, Clone)]
pub struct EnumInfo<'db> {
    /// (variant_name, variant_index, variant_data)
    pub variants: Vec<(Symbol<'db>, usize, VariantInfo<'db>)>,
}

pub struct ExprLowerer<'db> {
    pub(crate) db: &'db dyn scrap_shared::Db,
    /// Source text for extracting literal values
    pub(crate) source: &'db str,
    /// Type table from type checking
    pub(crate) type_table: scrap_tycheck::TypeTable<'db>,
    /// All local variable declarations (params + named locals + temps)
    pub local_decls: Vec<ir::LocalDecl<'db>>,
    /// CFG builder for managing basic blocks
    pub cfg_builder: BasicBlockBuilder<'db>,
    /// Symbol table mapping names to local IDs
    pub(crate) symbol_table: HashMap<Symbol<'db>, ir::LocalId>,
    /// Struct field name → index mapping for field access resolution.
    /// Key: struct name, Value: map of field_name Symbol → field_index
    pub(crate) struct_fields: HashMap<String, HashMap<Symbol<'db>, usize>>,
    /// Enum name → variant info mapping.
    pub(crate) enum_info: HashMap<String, EnumInfo<'db>>,
}

impl<'db> ExprLowerer<'db> {
    /// Create a new expression lowerer with source text for literal extraction
    pub fn new(
        db: &'db dyn scrap_shared::Db,
        source: &'db str,
        type_table: scrap_tycheck::TypeTable<'db>,
    ) -> Self {
        Self {
            db,
            source,
            type_table,
            local_decls: Vec::new(),
            cfg_builder: BasicBlockBuilder::new(db),
            symbol_table: HashMap::new(),
            struct_fields: HashMap::new(),
            enum_info: HashMap::new(),
        }
    }

    /// Look up the type of an expression by its NodeId from the type table
    pub(crate) fn lookup_expr_type(&self, node_id: scrap_shared::NodeId) -> Option<&scrap_tycheck::ResolvedTy<'db>> {
        self.type_table.expr_type(self.db, node_id)
    }

    /// Look up the type of a local variable by its NodeId from the type table
    pub(crate) fn lookup_local_type(&self, node_id: scrap_shared::NodeId) -> Option<&scrap_tycheck::ResolvedTy<'db>> {
        self.type_table.local_type(self.db, node_id)
    }

    /// Look up the type of an expression from type table and convert to IR type.
    /// Returns Bool as fallback for tests that don't populate the type table.
    pub(crate) fn lookup_and_convert_type(&self, node_id: scrap_shared::NodeId) -> ir::Ty<'db> {
        self.lookup_expr_type(node_id)
            .map(|resolved| crate::ty_convert::resolved_to_ir(self.db, resolved))
            .unwrap_or(ir::Ty::Bool) // Fallback for tests
    }

    /// Look up the type of a local variable from type table and convert to IR type.
    /// Returns Bool as fallback for tests that don't populate the type table.
    pub(crate) fn lookup_and_convert_local_type(&self, node_id: scrap_shared::NodeId) -> ir::Ty<'db> {
        self.lookup_local_type(node_id)
            .map(|resolved| crate::ty_convert::resolved_to_ir(self.db, resolved))
            .unwrap_or(ir::Ty::Bool) // Fallback for tests
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

    /// The return place is always _0
    pub fn return_place(&self) -> ir::Place<'db> {
        ir::Place::Local(ir::LocalId(0))
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
