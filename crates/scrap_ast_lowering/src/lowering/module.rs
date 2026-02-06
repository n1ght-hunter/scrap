//! Module and function lowering from AST to IR

use scrap_ast::{
    fndef::FnDef,
    item::{Item, ItemKind},
    local::LocalKind,
    pat::PatKind,
    stmt::StmtKind,
};
use scrap_ir as ir;
use scrap_shared::id::ModuleId;

use crate::{lowerer::ExprLowerer, lowering::lower_type, MResult};

/// Lower a module with its items
pub fn lower_module<'db>(
    db: &'db dyn scrap_shared::Db,
    module_id: ModuleId<'db>,
    ast_items: &[Item<'db>],
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
) -> MResult<ir::Module<'db>> {
    let mut items = Vec::new();

    for item in ast_items {
        match &item.kind {
            ItemKind::Fn(fn_def) => {
                let mir_function = lower_function(db, *fn_def, source, type_table)?;
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
pub fn lower_function<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
) -> MResult<ir::Function<'db>> {
    let signature = lower_signature(db, ast_function)?;
    let body = lower_body(db, ast_function, source, type_table)?;

    Ok(ir::Function::new(db, signature, body))
}

/// Lower function signature
pub fn lower_signature<'db>(
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
        Some(ty) => lower_type(db, ty)?,
        None => ir::Ty::Void,
    };

    Ok(ir::Signature::new(db, name, params, return_ty))
}

/// Lower function body using ExprLowerer for proper expression handling
pub fn lower_body<'db>(
    db: &'db dyn scrap_shared::Db,
    ast_function: FnDef<'db>,
    source: &'db str,
    type_table: scrap_tycheck::TypeTable<'db>,
) -> MResult<ir::Body<'db>> {
    let mut lowerer = ExprLowerer::new(db, source, type_table);

    // 1. Register function parameters as local variables
    for param in ast_function.args(db).iter() {
        let param_ty = lower_type(db, &*param.ty)?;
        let local_id = lowerer.allocate_named_local(param.ident.name, param_ty);
        lowerer.insert_binding(param.ident.name, local_id);
    }

    // 2. Process all statements in the body
    let body = ast_function.body(db);
    for stmt in &body.stmts {
        match &stmt.kind {
            StmtKind::Let(local) => {
                // Handle let bindings
                if let PatKind::Ident(_, ident, _) = &local.pat.kind {
                    // Get type from explicit annotation or type table
                    let ty = if let Some(explicit_ty) = local.ty.as_ref() {
                        lower_type(db, explicit_ty)?
                    } else {
                        // No explicit type - look up from type table using pattern NodeId
                        lowerer.lookup_and_convert_type(local.pat.id)
                    };

                    let local_id = lowerer.allocate_named_local(ident.name, ty);
                    lowerer.insert_binding(ident.name, local_id);

                    // If there's an initializer, lower it and emit assignment
                    if let LocalKind::Init(init) = &local.kind {
                        let rhs = lowerer.lower_expr(init)?;
                        lowerer.emit_assign(ir::Place::Local(local_id), ir::Rvalue::Use(rhs));
                    }
                }
            }
            StmtKind::Semi(expr) | StmtKind::Expr(expr) => {
                // Lower the expression (handles returns, control flow, etc.)
                lowerer.lower_expr(expr)?;
            }
            StmtKind::Item(_) | StmtKind::Empty => {
                // Skip items and empty statements
            }
        }
    }

    // 3. Ensure the final block is terminated
    if !lowerer.cfg_builder.current_block_is_terminated() {
        lowerer.cfg_builder.finish_block(ir::Terminator::Return);
    }

    // 4. Build the CFG and return the body
    let blocks = lowerer.cfg_builder.build();
    Ok(ir::Body::new(db, blocks, lowerer.local_decls))
}
