# NodeId in Rust AST Structures

Based on the official Rust compiler documentation and source code, here's a comprehensive overview of which AST structures have NodeIds.

## Official Rust Documentation

> **Every node in the AST has its own `NodeId`, including top-level items such as structs, but also individual statements and expressions. A `NodeId` is an identifier number that uniquely identifies an AST node within a crate.**
>
> — *The Rust Compiler Developer Guide*

## Core NodeId Trait

```rust
/// A trait for AST nodes having an ID.
pub trait HasNodeId {
    fn node_id(&self) -> NodeId;
    fn node_id_mut(&mut self) -> &mut NodeId;
}
```

## AST Structures with NodeIds

### Top-Level Items
According to the Rust compiler source (`rustc_ast/src/ast_traits.rs`), the following structures implement `HasNodeId`:

```rust
impl_has_node_id!(
    Arm,              // Match arms
    AssocItem,        // Associated items (methods, types, consts in traits/impls)
    Crate,            // Root crate node
    Expr,             // All expressions
    ExprField,        // Struct/enum field expressions
    FieldDef,         // Field definitions in structs/enums
    ForeignItem,      // Items in extern blocks
    GenericParam,     // Generic parameters (T, 'a, const N: usize)
    Item,             // Top-level items (fn, struct, enum, etc.)
    Param,            // Function parameters
    Pat,              // Patterns
    PatField,         // Pattern fields in destructuring
    Stmt,             // Statements
    Ty,               // Type expressions
    Variant,          // Enum variants
    WherePredicate,   // Where clause predicates
);
```

### AstIdNode Types (Rust-Analyzer)
In Rust-Analyzer's implementation, these nodes get special AST IDs:

**Named Items:**
- `Enum` = name
- `Struct` = name  
- `Union` = name
- `ExternCrate` = name_ref
- `MacroDef` = name
- `MacroRules` = name
- `Module` = name
- `Static` = name
- `Trait` = name
- `TraitAlias` = name

**Associated Items:**
- `Variant` = name()
- `Const` = name()
- `Fn` = name()
- `MacroCall` = path()
- `TypeAlias` = name()

**Special Cases:**
- `ExternBlock` - External blocks
- `Use` - Use declarations
- `Impl` - Implementation blocks
- `AsmExpr` - Inline assembly expressions

## Documentation from Rust Developer Guide

### NodeId Purpose
> **NodeIds are used in all the rustc bits that operate directly on the AST, like macro expansion and name resolution.**

### Important Notes
> **However, because they are absolute within a crate, adding or removing a single node in the AST causes all the subsequent NodeIds to change. This renders NodeIds pretty much useless for incremental compilation, where you want as few things as possible to change.**

### DefId Relationship
> **If you are creating new DefIds, since each DefId needs to have a corresponding NodeId, it is advisable to add these NodeIds to the AST so you don't have to generate new ones during lowering.**

## AST ID Strategy for Incremental Compilation

From Rust-Analyzer's approach, we can see a sophisticated strategy for stable IDs:

```rust
/// Maps items' `SyntaxNode`s to `ErasedFileAstId`s and back.
/// 
/// Specifically, it enumerates all items in a file and uses position of an
/// item as an ID. That way, id's don't change unless the set of items itself
/// changes.
```

**Key Principles:**
1. **Top-level items** get stable IDs based on name and kind
2. **Associated items** (methods, consts in impls/traits) get IDs with parent context
3. **Blocks** only get IDs if they contain items
4. **Expressions and statements** get NodeIDs for compilation phases

## Testing Coverage

From the Rust compiler tests, these node types are verified to have proper ID allocation:

```rust
SyntaxKind::EXTERN_CRATE
| SyntaxKind::FN
| SyntaxKind::UNION  
| SyntaxKind::STRUCT
| SyntaxKind::MACRO_RULES
| SyntaxKind::MACRO_DEF
| SyntaxKind::MACRO_CALL
| SyntaxKind::TRAIT
| SyntaxKind::IMPL
| SyntaxKind::TYPE_ALIAS
| SyntaxKind::ENUM
| SyntaxKind::VARIANT
| SyntaxKind::EXTERN_BLOCK
| SyntaxKind::STATIC
| SyntaxKind::CONST
```

## Implementation Guidelines

For your Scrap language implementation:

1. **Every AST node should have a NodeId** - This includes expressions, statements, patterns, types, and items
2. **Use atomic counters** for thread-safe unique ID generation
3. **Include NodeIds in**:
   - All expression types (`Expr`, `Lit`, etc.)
   - All item definitions (`FnDef`, `StructDef`, `EnumDef`)
   - All patterns (`Pat`)
   - Type references (`Type`, `Ty`)
   - Identifiers (`Ident`)
   - Fields and parameters (`Field`, `Param`)
   - Statements and blocks (`Stmt`, `Block`)

## Conclusion

The Rust compiler assigns NodeIds to virtually every AST construct to enable:
- **Macro expansion** tracking
- **Name resolution** mapping
- **Error reporting** with precise locations
- **DefId generation** in later compiler phases
- **Incremental compilation** support (with sophisticated strategies)

Your implementation should follow this pattern to ensure compatibility with standard compiler techniques and enable advanced features like incremental compilation and precise error reporting.
