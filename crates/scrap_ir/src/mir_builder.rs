use rayon::prelude::*;
use scrap_ast::{
    block::Block,
    expr::{Expr, ExprKind},
    fndef::FnDef,
    item::{Item, ItemKind},
    pat::PatKind,
    stmt::{Stmt, StmtKind},
    typedef::Ty,
};

use crate::mir;


#[derive(Debug, Clone, thiserror::Error)]
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
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct MirBuilder {}

impl MirBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn lower_can(&self, ast_modules: Vec<(String, Vec<Item>)>) -> Result<mir::Can> {
        let modules = ast_modules.into_par_iter()
            .map(|(path, items)| self.lower_module(path, &items))
            .collect::<Result<Vec<_>>>()?;


        Ok(mir::Can { modules })
    }

    pub fn lower_module(&self, path: String, ast_items: &[Item]) -> Result<mir::Module> {
        let mut module = mir::Module {
            path,
            items: vec![],
        };
        for item in ast_items {
            match &item.kind {
                ItemKind::Fn(fn_def) => {
                    let mir_function = self.lower_function(fn_def)?;
                    module.items.push(mir::Items::Function(mir_function));
                }
                _ => {
                    return Err(BuilderError::LowerModuleError);
                }
            }
        }
        Ok(module)
    }

    /// The main entry point.
    pub fn lower_function(&self, ast_function: &FnDef) -> Result<mir::Function> {
        Ok(mir::Function {
            signature: self.lower_signature(&ast_function)?,
            body: self.lower_body(&ast_function.body)?,
        })
    }

    fn lower_body(&self, ast_block: &Block) -> Result<mir::Body> {
        let mut body = mir::Body::default();
        let mut current_block = mir::BasicBlock {
            statements: vec![],
            terminator: mir::Terminator::Unreachable,
        };
        for stmt in &*ast_block.stmts {
            match &stmt.kind {
                StmtKind::Let(local) => {
                    if let PatKind::Ident(_, ident, pat) = &local.pat.kind {
                        let ty = local
                            .ty
                            .as_ref()
                            .map_or(Ok(mir::Ty::Infer), |t| self.lower_type(t))?;
                        body.local_decls.push(mir::LocalDecl {
                            name: Some(ident.name.clone()),
                            ty,
                        });
                    } else {
                        return Err(BuilderError::LowerBodyError);
                    }
                }
                StmtKind::Semi(expr) => {
                    match &expr.kind {
                        ExprKind::Return(_) => {
                            // Handle return expression
                            current_block.terminator = mir::Terminator::Return;
                            body.blocks.push(std::mem::take(&mut current_block));
                        }
                        _ => {
                            // Other expressions can be handled here
                        }
                    }
                }
                _ => return Err(BuilderError::LowerBodyError),
            }
        }
        Ok(body)
    }

    fn lower_signature(&self, ast_function: &FnDef) -> Result<mir::Signature> {
        Ok(mir::Signature {
            name: ast_function.ident.name.clone(),
            params: ast_function
                .args
                .iter()
                .map(|arg| {
                    self.lower_type(&arg.ty)
                        .map(|t| (arg.ident.name.clone(), t))
                })
                .collect::<Result<_>>()?,
            return_ty: match ast_function.ret_type.as_ref() {
                Some(ty) => Some(self.lower_type(ty)?),
                None => None,
            },
        })
    }

    fn lower_type(&self, ast_type: &Ty) -> Result<mir::Ty> {
        match ast_type.kind {
            "int" => Ok(mir::Ty::Int),
            "bool" => Ok(mir::Ty::Bool),
            "String" => Ok(mir::Ty::Str),
            _ => Ok(mir::Ty::Adt(mir::Resolved::Unresolved(
                ast_type.ident.name.clone(),
            ))),
        }
    }
}
