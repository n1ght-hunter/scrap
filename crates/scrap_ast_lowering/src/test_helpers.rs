//! Test helper functions for constructing AST nodes
//!
//! This module provides convenient builder functions to reduce boilerplate
//! when writing tests for AST lowering.

use scrap_ast::{
    block::Block,
    expr::{Expr, ExprKind},
    lit::{Lit, LitKind},
    operators::{AssignOp, AssignOpKind, BinOp, BinOpKind},
    stmt::{Stmt, StmtKind},
};
use scrap_shared::{
    ident::{Ident, Symbol},
    path::{Path, PathSegment},
    NodeId,
};
use scrap_span::Span;
use thin_vec::ThinVec;

/// Create a simple span for testing
pub fn test_span(db: &dyn scrap_shared::Db) -> Span<'_> {
    Span::new(db, 0, 0)
}

/// Create a simple node ID for testing
pub fn test_node_id() -> NodeId {
    NodeId::new(0, 0)
}

/// Create an integer literal expression
pub fn create_int_lit<'db>(db: &'db dyn scrap_shared::Db, _value: i64) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    let lit = Lit {
        id: node_id,
        kind: LitKind::Integer,
        span,
    };

    Expr {
        id: node_id,
        kind: ExprKind::Lit(lit),
        span,
    }
}

/// Create a boolean literal expression
pub fn create_bool_lit<'db>(db: &'db dyn scrap_shared::Db, _value: bool) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    let lit = Lit {
        id: node_id,
        kind: LitKind::Bool,
        span,
    };

    Expr {
        id: node_id,
        kind: ExprKind::Lit(lit),
        span,
    }
}

/// Create a string literal expression
pub fn create_string_lit<'db>(db: &'db dyn scrap_shared::Db, _value: &str) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    let lit = Lit {
        id: node_id,
        kind: LitKind::Str,
        span,
    };

    Expr {
        id: node_id,
        kind: ExprKind::Lit(lit),
        span,
    }
}

/// Create a float literal expression
pub fn create_float_lit<'db>(db: &'db dyn scrap_shared::Db, _value: f64) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    let lit = Lit {
        id: node_id,
        kind: LitKind::Float,
        span,
    };

    Expr {
        id: node_id,
        kind: ExprKind::Lit(lit),
        span,
    }
}

/// Create an identifier expression (variable reference)
pub fn create_ident_expr<'db>(db: &'db dyn scrap_shared::Db, name: &str) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    let symbol = Symbol::new(db, name.to_string());
    let ident = Ident {
        id: node_id,
        name: symbol,
        span,
    };

    let path = Path {
        span,
        segments: ThinVec::from([PathSegment {
            ident,
            id: node_id,
        }]),
    };

    Expr {
        id: node_id,
        kind: ExprKind::Path(path),
        span,
    }
}

/// Create a binary operation expression
pub fn create_binary_expr<'db>(
    db: &'db dyn scrap_shared::Db,
    op_kind: BinOpKind,
    lhs: Expr<'db>,
    rhs: Expr<'db>,
) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    let op = BinOp {
        node: op_kind,
        span,
    };

    Expr {
        id: node_id,
        kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)),
        span,
    }
}

/// Create a parenthesized expression
pub fn create_paren_expr<'db>(db: &'db dyn scrap_shared::Db, inner: Expr<'db>) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Expr {
        id: node_id,
        kind: ExprKind::Paren(Box::new(inner)),
        span,
    }
}

/// Create an empty block
pub fn create_empty_block<'db>(db: &'db dyn scrap_shared::Db) -> Block<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Block {
        stmts: ThinVec::new(),
        id: node_id,
        span,
    }
}

/// Create an assignment expression: lhs = rhs
pub fn create_assign_expr<'db>(
    db: &'db dyn scrap_shared::Db,
    lhs: Expr<'db>,
    rhs: Expr<'db>,
) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Expr {
        id: node_id,
        kind: ExprKind::Assign(Box::new(lhs), Box::new(rhs), span),
        span,
    }
}

/// Create a compound assignment expression: lhs op= rhs
pub fn create_assign_op_expr<'db>(
    db: &'db dyn scrap_shared::Db,
    op_kind: AssignOpKind,
    lhs: Expr<'db>,
    rhs: Expr<'db>,
) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    let op = AssignOp {
        node: op_kind,
        span,
    };

    Expr {
        id: node_id,
        kind: ExprKind::AssignOp(op, Box::new(lhs), Box::new(rhs)),
        span,
    }
}

/// Create a return expression
pub fn create_return_expr<'db>(db: &'db dyn scrap_shared::Db, value: Option<Expr<'db>>) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Expr {
        id: node_id,
        kind: ExprKind::Return(value.map(Box::new)),
        span,
    }
}

/// Create an if expression without else
pub fn create_if_expr<'db>(
    db: &'db dyn scrap_shared::Db,
    cond: Expr<'db>,
    then_block: Block<'db>,
) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Expr {
        id: node_id,
        kind: ExprKind::If(Box::new(cond), Box::new(then_block), None),
        span,
    }
}

/// Create an if-else expression
pub fn create_if_else_expr<'db>(
    db: &'db dyn scrap_shared::Db,
    cond: Expr<'db>,
    then_block: Block<'db>,
    else_expr: Expr<'db>,
) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Expr {
        id: node_id,
        kind: ExprKind::If(Box::new(cond), Box::new(then_block), Some(Box::new(else_expr))),
        span,
    }
}

/// Create a block with statements
pub fn create_block<'db>(db: &'db dyn scrap_shared::Db, stmts: Vec<Stmt<'db>>) -> Block<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Block {
        stmts: ThinVec::from(stmts),
        id: node_id,
        span,
    }
}

/// Create a statement with semicolon
pub fn create_semi_stmt<'db>(db: &'db dyn scrap_shared::Db, expr: Expr<'db>) -> Stmt<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Stmt {
        id: node_id,
        kind: StmtKind::Semi(Box::new(expr)),
        span,
    }
}

/// Create an expression statement (without semicolon)
pub fn create_expr_stmt<'db>(db: &'db dyn scrap_shared::Db, expr: Expr<'db>) -> Stmt<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Stmt {
        id: node_id,
        kind: StmtKind::Expr(Box::new(expr)),
        span,
    }
}

/// Create an array literal expression
pub fn create_array_expr<'db>(db: &'db dyn scrap_shared::Db, elements: Vec<Expr<'db>>) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Expr {
        id: node_id,
        kind: ExprKind::Array(ThinVec::from(elements.into_iter().map(Box::new).collect::<Vec<_>>())),
        span,
    }
}

/// Create a function call expression
pub fn create_call_expr<'db>(
    db: &'db dyn scrap_shared::Db,
    func: Expr<'db>,
    args: Vec<Expr<'db>>,
) -> Expr<'db> {
    let span = test_span(db);
    let node_id = test_node_id();

    Expr {
        id: node_id,
        kind: ExprKind::Call(
            Box::new(func),
            ThinVec::from(args.into_iter().map(Box::new).collect::<Vec<_>>()),
        ),
        span,
    }
}
