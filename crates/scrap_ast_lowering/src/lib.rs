use scrap_ast::{
    block::Block,
    expr::ExprKind,
    fndef::FnDef,
    item::{Item, ItemKind},
    pat::PatKind,
    stmt::StmtKind,
    typedef::{Ty, TyKind},
};
use scrap_span::Symbol;
use scrap_ir as ir;

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum BuilderError {
    #[error("Failed to lower CAN")]
    LowerCanError,
    #[error("Failed to lower module")]
    LowerModuleError,
    #[error("Failed to lower function")]
    LowerFunctionError,
    #[error("Failed to lower body")]
    LowerBodyError,
    #[error("Failed to lower signature")]
    LowerSignatureError,
    #[error("Failed to lower type")]
    LowerTypeError,
}

type Error = BuilderError;
type MResult<T> = std::result::Result<T, Error>;

/// Result wrapper for lowered IR
#[salsa::tracked(debug, persist)]
pub struct LoweredIr<'db> {
    pub can: ir::Can<'db>,
}

/// Main entry point: lower parsed AST modules to IR
pub fn lower_to_ir<'db>(
    db: &'db dyn salsa::Database,
    modules: Vec<(String, Vec<Item<'db>>)>,
) -> Result<LoweredIr<'db>, BuilderError> {
    let mut mir_modules = Vec::new();

    for (path, items) in modules {
        let module = lower_module(db, path, &items)?;
        mir_modules.push(module);
    }

    let can = ir::Can::new(db, mir_modules);
    Ok(LoweredIr::new(db, can))
}

/// Lower a module with its items
fn lower_module<'db>(
    db: &'db dyn salsa::Database,
    path: String,
    ast_items: &[Item<'db>],
) -> MResult<ir::Module<'db>> {
    let path_symbol = Symbol::new(db, path);
    let mut items = Vec::new();

    for item in ast_items {
        match &item.kind {
            ItemKind::Fn(fn_def) => {
                let mir_function = lower_function(db, *fn_def)?;
                items.push(ir::Items::Function(mir_function));
            }
            _ => {
                // Skip non-function items for now
                continue;
            }
        }
    }

    Ok(ir::Module::new(db, path_symbol, items))
}

/// Lower a function definition
fn lower_function<'db>(
    db: &'db dyn salsa::Database,
    ast_function: FnDef<'db>,
) -> MResult<ir::Function<'db>> {
    let signature = lower_signature(db, ast_function)?;
    let body = lower_body(db, ast_function.body(db))?;

    Ok(ir::Function::new(db, signature, body))
}

/// Lower function signature
fn lower_signature<'db>(
    db: &'db dyn salsa::Database,
    ast_function: FnDef<'db>,
) -> MResult<ir::Signature<'db>> {
    let name = ast_function.ident(db).name;

    let mut params = Vec::new();
    for arg in ast_function.args(db).iter() {
        let param_name = arg.ident.name;
        let param_ty = lower_type(db, &*arg.ty)?;
        params.push((param_name, param_ty));
    }

    let return_ty = match ast_function.ret_type(db).as_ref() {
        Some(ty) => Some(lower_type(db, ty)?),
        None => None,
    };

    Ok(ir::Signature::new(db, name, params, return_ty))
}

/// Lower function body
fn lower_body<'db>(db: &'db dyn salsa::Database, ast_block: &Block<'db>) -> MResult<ir::Body<'db>> {
    let mut blocks = Vec::new();
    let mut local_decls = Vec::new();
    let statements = Vec::new();
    let mut terminator = ir::Terminator::Unreachable;

    for stmt in &ast_block.stmts {
        match &stmt.kind {
            StmtKind::Let(local) => {
                if let PatKind::Ident(_, ident, _pat) = &local.pat.kind {
                    let ty = local
                        .ty
                        .as_ref()
                        .map_or(Ok(ir::Ty::Infer), |t| lower_type(db, t))?;

                    let local_decl = ir::LocalDecl::new(db, Some(ident.name), ty);
                    local_decls.push(local_decl);
                } else {
                    return Err(BuilderError::LowerBodyError);
                }
            }
            StmtKind::Semi(expr) => {
                match &expr.kind {
                    ExprKind::Return(_) => {
                        terminator = ir::Terminator::Return;
                    }
                    _ => {
                        // Other expressions can be handled here
                    }
                }
            }
            _ => return Err(BuilderError::LowerBodyError),
        }
    }

    // Create the basic block
    let basic_block = ir::BasicBlock::new(db, statements, terminator);
    blocks.push(basic_block);

    Ok(ir::Body::new(db, blocks, local_decls))
}

/// Lower a type from AST to IR
fn lower_type<'db>(db: &'db dyn salsa::Database, ast_type: &Ty<'db>) -> MResult<ir::Ty<'db>> {
    match &ast_type.kind {
        TyKind::Path(path) => {
            // Get the last segment as the type name
            let type_name = path
                .single_segment()
                .map(|e| e.ident.name.text(db).as_str())
                .unwrap_or("");

            match type_name {
                "int" => Ok(ir::Ty::Int),
                "bool" => Ok(ir::Ty::Bool),
                "String" => Ok(ir::Ty::Str),
                _ => {
                    let type_id = ir::TypeId::new(db, type_name);
                    Ok(ir::Ty::Adt(type_id))
                }
            }
        }
        TyKind::Never => Ok(ir::Ty::Never),
        _ => Ok(ir::Ty::Infer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scrap_ast::{
        Visibility, VisibilityKind,
        block::Block,
        fndef::FnDef,
        ident::Ident,
        item::Item,
        pat::{BindingMode, ByRef, Pat, PatKind},
        path::Path,
        typedef::Ty,
    };
    use scrap_shared::{Mutability, NodeId, salsa::ScrapDb};
    use scrap_span::{Span, Symbol};
    use thin_vec::ThinVec;

    /// Test helper that wraps the logic in a Salsa tracked function
    #[salsa::tracked]
    fn test_lower_simple_function_impl<'db>(db: &'db dyn salsa::Database) -> bool {
        let span = Span::new(db, 0, 0);
        let node_id = NodeId::new(0, 0);
        let name = Symbol::new(db, "test_fn".to_string());
        let ident = Ident {
            id: node_id,
            name,
            span,
        };

        let body = Block {
            stmts: ThinVec::new(),
            id: node_id,
            span,
        };
        let fn_def = FnDef::new(db, node_id, ident, ThinVec::new(), None, body, span);

        let result = lower_function(db, fn_def);
        if result.is_err() {
            return false;
        }

        let function = result.unwrap();
        function.signature(db).name(db).text(db) == "test_fn"
            && function.signature(db).params(db).len() == 0
            && function.signature(db).return_ty(db).is_none()
    }

    #[test]
    fn test_lower_simple_function() {
        let db = ScrapDb::default();
        assert!(test_lower_simple_function_impl(&db));
    }

    #[salsa::tracked]
    fn test_lower_function_with_params_impl<'db>(db: &'db dyn salsa::Database) -> bool {
        let span = Span::new(db, 0, 0);
        let node_id = NodeId::new(0, 0);

        let name = Symbol::new(db, "add".to_string());
        let ident = Ident {
            id: node_id,
            name,
            span,
        };

        // Create parameter 'a: int'
        let a_name = Symbol::new(db, "a".to_string());
        let a_ident = Ident {
            id: node_id,
            name: a_name,
            span,
        };
        let int_sym = Symbol::new(db, "int".to_string());
        let int_path = Path {
            span,
            segments: ThinVec::from([scrap_ast::path::PathSegment {
                ident: Ident {
                    id: node_id,
                    name: int_sym,
                    span,
                },
                id: node_id,
            }]),
        };
        let param_a = scrap_ast::fndef::Param {
            id: node_id,
            ident: a_ident,
            ty: Box::new(Ty {
                id: node_id,
                kind: scrap_ast::typedef::TyKind::Path(int_path.clone()),
                span,
            }),
            pat: Box::new(Pat {
                id: node_id,
                kind: PatKind::Ident(BindingMode(ByRef::No, Mutability::Not), a_ident, None),
                span,
            }),
            span,
        };

        // Create parameter 'b: int'
        let b_name = Symbol::new(db, "b".to_string());
        let b_ident = Ident {
            id: node_id,
            name: b_name,
            span,
        };
        let param_b = scrap_ast::fndef::Param {
            id: node_id,
            ident: b_ident,
            ty: Box::new(Ty {
                id: node_id,
                kind: scrap_ast::typedef::TyKind::Path(int_path),
                span,
            }),
            pat: Box::new(Pat {
                id: node_id,
                kind: PatKind::Ident(BindingMode(ByRef::No, Mutability::Not), b_ident, None),
                span,
            }),
            span,
        };

        let body = Block {
            stmts: ThinVec::new(),
            id: node_id,
            span,
        };
        let args = ThinVec::from([param_a, param_b]);
        let fn_def = FnDef::new(db, node_id, ident, args, None, body, span);

        let result = lower_function(db, fn_def);
        if result.is_err() {
            return false;
        }

        let function = result.unwrap();
        let signature = function.signature(db);
        signature.name(db).text(db) == "add"
            && signature.params(db).len() == 2
            && signature.params(db)[0].0.text(db) == "a"
            && signature.params(db)[0].1 == ir::Ty::Int
            && signature.params(db)[1].0.text(db) == "b"
            && signature.params(db)[1].1 == ir::Ty::Int
    }

    #[test]
    fn test_lower_function_with_params() {
        let db = ScrapDb::default();
        assert!(test_lower_function_with_params_impl(&db));
    }

    #[salsa::tracked]
    fn test_lower_type_primitives_impl<'db>(db: &'db dyn salsa::Database) -> bool {
        let span = Span::new(db, 0, 0);
        let node_id = NodeId::new(0, 0);

        // Test int type
        let int_name = Symbol::new(db, "int".to_string());
        let int_path = Path {
            span,
            segments: ThinVec::from([scrap_ast::path::PathSegment {
                ident: Ident {
                    id: node_id,
                    name: int_name,
                    span,
                },
                id: node_id,
            }]),
        };
        let int_ty = Ty {
            id: node_id,
            kind: scrap_ast::typedef::TyKind::Path(int_path),
            span,
        };
        if lower_type(db, &int_ty).unwrap() != ir::Ty::Int {
            return false;
        }

        // Test bool type
        let bool_name = Symbol::new(db, "bool".to_string());
        let bool_path = Path {
            span,
            segments: ThinVec::from([scrap_ast::path::PathSegment {
                ident: Ident {
                    id: node_id,
                    name: bool_name,
                    span,
                },
                id: node_id,
            }]),
        };
        let bool_ty = Ty {
            id: node_id,
            kind: scrap_ast::typedef::TyKind::Path(bool_path),
            span,
        };
        if lower_type(db, &bool_ty).unwrap() != ir::Ty::Bool {
            return false;
        }

        // Test String type
        let string_name = Symbol::new(db, "String".to_string());
        let string_path = Path {
            span,
            segments: ThinVec::from([scrap_ast::path::PathSegment {
                ident: Ident {
                    id: node_id,
                    name: string_name,
                    span,
                },
                id: node_id,
            }]),
        };
        let string_ty = Ty {
            id: node_id,
            kind: scrap_ast::typedef::TyKind::Path(string_path),
            span,
        };
        lower_type(db, &string_ty).unwrap() == ir::Ty::Str
    }

    #[test]
    fn test_lower_type_primitives() {
        let db = ScrapDb::default();
        assert!(test_lower_type_primitives_impl(&db));
    }

    #[salsa::tracked]
    fn test_lower_module_impl<'db>(db: &'db dyn salsa::Database) -> bool {
        let span = Span::new(db, 0, 0);
        let node_id = NodeId::new(0, 0);

        let name = Symbol::new(db, "module_fn".to_string());
        let ident = Ident {
            id: node_id,
            name,
            span,
        };
        let body = Block {
            stmts: ThinVec::new(),
            id: node_id,
            span,
        };
        let fn_def = FnDef::new(db, node_id, ident, ThinVec::new(), None, body, span);

        let item = Item {
            kind: scrap_ast::item::ItemKind::Fn(fn_def),
            span,
            id: node_id,
            vis: Visibility {
                kind: VisibilityKind::Public,
                span,
            },
        };

        let result = lower_module(db, "test_module".to_string(), &[item]);
        if result.is_err() {
            return false;
        }

        let module = result.unwrap();
        module.path(db).text(db) == "test_module" && module.items(db).len() == 1
    }

    #[test]
    fn test_lower_module() {
        let db = ScrapDb::default();
        assert!(test_lower_module_impl(&db));
    }

    #[salsa::tracked]
    fn test_lower_empty_module_impl<'db>(db: &'db dyn salsa::Database) -> bool {
        let result = lower_module(db, "empty_module".to_string(), &[]);
        if result.is_err() {
            return false;
        }

        let module = result.unwrap();
        module.path(db).text(db) == "empty_module" && module.items(db).len() == 0
    }

    #[test]
    fn test_lower_empty_module() {
        let db = ScrapDb::default();
        assert!(test_lower_empty_module_impl(&db));
    }

    #[test]
    fn test_builder_error_display() {
        let error = BuilderError::LowerCanError;
        assert_eq!(error.to_string(), "Failed to lower CAN");

        let error = BuilderError::LowerTypeError;
        assert_eq!(error.to_string(), "Failed to lower type");
    }
}
