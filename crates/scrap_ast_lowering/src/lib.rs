mod cfg_builder;
mod expr_lowering;
#[cfg(test)]
mod test_helpers;

use scrap_ast::{
    block::Block,
    expr::ExprKind,
    fndef::FnDef,
    item::{Item, ItemKind},
    pat::PatKind,
    stmt::StmtKind,
    typedef::{Ty, TyKind},
};
use scrap_ir as ir;
use scrap_shared::id::ModuleId;

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
    #[error("Failed to lower expression")]
    LowerExpressionError,
}

type Error = BuilderError;
type MResult<T> = std::result::Result<T, Error>;

/// Result wrapper for lowered IR
#[salsa::tracked(debug, persist)]
pub struct LoweredIr<'db> {
    pub can: ir::Can<'db>,
}

/// Lower a single parsed file to an IR module (tracked function for parallelization)
#[salsa::tracked(persist)]
pub fn lower_parsed_file<'db>(
    db: &'db dyn scrap_shared::Db,
    file: scrap_parser::ParsedFile<'db>,
    module_id: ModuleId<'db>,
) -> Option<ir::Module<'db>> {
    let ast = file.ast(db);

    let items: Vec<Item<'db>> = match ast {
        scrap_parser::CanOrModule::Can(can) => {
            can.items(db).iter().map(|b| (**b).clone()).collect()
        }
        scrap_parser::CanOrModule::Module(module) => {
            if let scrap_ast::module::ModuleKind::Loaded(items, _, _) = module.kind(db) {
                items.iter().map(|b| (**b).clone()).collect()
            } else {
                eprintln!("Module '{}' is not loaded", module_id.path(db));
                return None;
            }
        }
    };

    match lower_module(db, module_id, &items) {
        Ok(module) => Some(module),
        Err(e) => {
            eprintln!("Error lowering module '{}': {}", module_id.path(db), e);
            None
        }
    }
}

/// Lower a module with its items
fn lower_module<'db>(
    db: &'db dyn scrap_shared::Db,
    module_id: ModuleId<'db>,
    ast_items: &[Item<'db>],
) -> MResult<ir::Module<'db>> {
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

    Ok(ir::Module::new(db, module_id, items))
}

/// Lower a function definition
fn lower_function<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
) -> MResult<ir::Function<'db>> {
    let signature = lower_signature(db, ast_function)?;
    let body = lower_body(db, ast_function.body(db))?;

    Ok(ir::Function::new(db, signature, body))
}

/// Lower function signature
fn lower_signature<'db>(
    db: &'db dyn scrap_shared::Db,
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
fn lower_body<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_block: &Block<'db>,
) -> MResult<ir::Body<'db>> {
    let mut blocks = Vec::new();
    let mut local_decls = Vec::new();
    let statements = Vec::new();
    let mut terminator = ir::Terminator::Unreachable;

    // Process statements
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
                    // For now, skip non-ident patterns (like wildcards, tuples, etc.)
                    continue;
                }
            }
            StmtKind::Semi(expr) => {
                match &expr.kind {
                    ExprKind::Return(_) => {
                        terminator = ir::Terminator::Return;
                    }
                    _ => {
                        // Other expressions - ignore for now
                    }
                }
            }
            StmtKind::Expr(expr) => {
                // Trailing expression - treat as implicit return
                match &expr.kind {
                    ExprKind::Return(_) => {
                        terminator = ir::Terminator::Return;
                    }
                    _ => {
                        // Other expressions as implicit return
                        terminator = ir::Terminator::Return;
                    }
                }
            }
            _ => {
                // Skip other statement kinds for now
                continue;
            }
        }
    }

    // If no explicit terminator, assume unreachable
    if matches!(terminator, ir::Terminator::Unreachable) && !ast_block.stmts.is_empty() {
        // If the last statement is an expression, treat as return
        if let Some(last_stmt) = ast_block.stmts.last() {
            if matches!(last_stmt.kind, StmtKind::Expr(_)) {
                terminator = ir::Terminator::Return;
            }
        }
    }

    // Create the basic block
    let basic_block = ir::BasicBlock::new(db, statements, terminator);
    blocks.push(basic_block);

    Ok(ir::Body::new(db, blocks, local_decls))
}

/// Lower a type from AST to IR
fn lower_type<'db>(db: &'db dyn scrap_shared::Db, ast_type: &Ty<'db>) -> MResult<ir::Ty<'db>> {
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
        item::Item,
        pat::{BindingMode, ByRef, Pat, PatKind},
        typedef::Ty,
    };
    use scrap_shared::{
        Mutability, NodeId,
        ident::{Ident, Symbol},
        path::Path,
        salsa::ScrapDb,
    };
    use scrap_span::Span;
    use thin_vec::ThinVec;

    /// Test helper that wraps the logic in a Salsa tracked function
    #[salsa::tracked]
    fn test_lower_simple_function_impl<'db>(db: &'db dyn scrap_shared::Db) -> bool {
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
    fn test_lower_function_with_params_impl<'db>(db: &'db dyn scrap_shared::Db) -> bool {
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
            segments: ThinVec::from([scrap_shared::path::PathSegment {
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
    fn test_lower_type_primitives_impl<'db>(db: &'db dyn scrap_shared::Db) -> bool {
        let span = Span::new(db, 0, 0);
        let node_id = NodeId::new(0, 0);

        // Test int type
        let int_name = Symbol::new(db, "int".to_string());
        let int_path = Path {
            span,
            segments: ThinVec::from([scrap_shared::path::PathSegment {
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
            segments: ThinVec::from([scrap_shared::path::PathSegment {
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
            segments: ThinVec::from([scrap_shared::path::PathSegment {
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
    #[scrap_macros::salsa_test]
    fn test_lower_module(db: &dyn scrap_shared::Db) {
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
        let module_id = ModuleId::new(db, Path::from_segment(db, "test_module"));

        let module = lower_module(db, module_id, &[item]).unwrap();

        assert_eq!(module.id(db), module_id);
        assert_eq!(module.items(db).len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_empty_module(db: &dyn scrap_shared::Db) {
        let module_id = ModuleId::new(db, Path::from_segment(db, "empty_module"));
        let module = lower_module(db, module_id, &[]).unwrap();

        assert_eq!(module.id(db), module_id);
        assert!(module.items(db).is_empty());
    }

    #[test]
    fn test_builder_error_display() {
        let error = BuilderError::LowerCanError;
        assert_eq!(error.to_string(), "Failed to lower CAN");

        let error = BuilderError::LowerTypeError;
        assert_eq!(error.to_string(), "Failed to lower type");
    }
}
