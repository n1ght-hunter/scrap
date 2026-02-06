mod cfg_builder;
mod lowerer;
mod lowering;
mod ty_convert;
#[cfg(test)]
mod test_helpers;

use scrap_ast::item::Item;
use scrap_ir as ir;
use scrap_shared::id::ModuleId;
use scrap_errors::ErrorGuaranteed;

pub use cfg_builder::BasicBlockBuilder;
pub use lowerer::ExprLowerer;
pub use lowering::{lower_body, lower_function, lower_module, lower_signature, lower_type};
pub use ty_convert::resolved_to_ir;

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
    #[error("Type error")]
    Error(ErrorGuaranteed),
}

type Error = BuilderError;
pub type MResult<T> = std::result::Result<T, Error>;

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
    type_table: scrap_tycheck::TypeTable<'db>,
) -> Option<ir::Module<'db>> {
    let ast = file.ast(db);
    let source = file.file(db).content(db);

    let items: Vec<Item<'db>> = match ast {
        scrap_parser::CanOrModule::Can(can) => {
            can.items(db).iter().map(|b| (**b).clone()).collect()
        }
        scrap_parser::CanOrModule::Module(module) => {
            if let scrap_ast::module::ModuleKind::Loaded(items, _, _) = module.kind(db) {
                items.iter().map(|b| (**b).clone()).collect()
            } else {
                eprintln!("Module '{}' is not loaded", module_id.path_str(db));
                return None;
            }
        }
    };

    match lower_module(db, module_id, &items, source, type_table) {
        Ok(module) => Some(module),
        Err(e) => {
            eprintln!("Error lowering module '{}': {}", module_id.path_str(db), e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_empty_type_table;
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

        let result = lower_function(db, fn_def, "", create_empty_type_table(db));
        if result.is_err() {
            return false;
        }

        let function = result.unwrap();
        function.signature(db).name(db).text(db) == "test_fn"
            && function.signature(db).params(db).len() == 0
            && function.signature(db).return_ty(db) == ir::Ty::Void
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

        let result = lower_function(db, fn_def, "", create_empty_type_table(db));
        if result.is_err() {
            return false;
        }

        let function = result.unwrap();
        let signature = function.signature(db);
        signature.name(db).text(db) == "add"
            && signature.params(db).len() == 2
            && signature.params(db)[0].0.text(db) == "a"
            && signature.params(db)[0].1 == ir::Ty::Int(scrap_shared::types::IntTy::I32)
            && signature.params(db)[1].0.text(db) == "b"
            && signature.params(db)[1].1 == ir::Ty::Int(scrap_shared::types::IntTy::I32)
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
        if lower_type(db, &int_ty).unwrap() != ir::Ty::Int(scrap_shared::types::IntTy::I32) {
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
        let path = scrap_shared::path::Path::from_segment(db, "test_module");
        let module_id = ModuleId::from_path(db, &path);

        let module = lower_module(db, module_id, &[item], "", create_empty_type_table(db)).unwrap();

        assert_eq!(module.id(db), module_id);
        assert_eq!(module.items(db).len(), 1);
    }

    #[scrap_macros::salsa_test]
    fn test_lower_empty_module(db: &dyn scrap_shared::Db) {
        let path = scrap_shared::path::Path::from_segment(db, "empty_module");
        let module_id = ModuleId::from_path(db, &path);
        let module = lower_module(db, module_id, &[], "", create_empty_type_table(db)).unwrap();

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
