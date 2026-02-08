use scrap_span::Span;
use strum_macros::{EnumDiscriminants, EnumIter};
use thin_vec::ThinVec;

use crate::{
    block::Block,
    lit::Lit,
    node_id::NodeId,
    operators::{AssignOp, BinOp, UnOp},
};
use scrap_shared::ident::Ident;
use scrap_shared::path::Path;

/// An expression node in the AST
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct Expr<'db> {
    pub id: NodeId,
    pub kind: ExprKind<'db>,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for Expr<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        self.kind.pretty_print_indent(f, indent)
    }
}

/// Expression kinds - subset of Rust's ExprKind enum
#[derive(
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    EnumDiscriminants,
    salsa::Update,
    serde::Serialize,
    serde::Deserialize,
)]
#[strum_discriminants(derive(EnumIter))]
pub enum ExprKind<'db> {
    /// An array literal (e.g., `[a, b, c, d]`)
    Array(ThinVec<Box<Expr<'db>>>),
    /// A function call
    Call(Box<Expr<'db>>, ThinVec<Box<Expr<'db>>>),
    /// A binary operation (e.g., `a + b`, `a * b`)
    Binary(BinOp<'db>, Box<Expr<'db>>, Box<Expr<'db>>),
    /// A literal value (e.g., `1`, `"foo"`)
    Lit(Lit<'db>),
    /// An `if` block, with an optional `else` block
    If(Box<Expr<'db>>, Box<Block<'db>>, Option<Box<Expr<'db>>>),
    /// A block (`{ ... }`)
    Block(Box<Block<'db>>),
    /// Variable reference
    Path(Path<'db>),
    /// A parenthesized expression
    Paren(Box<Expr<'db>>),
    /// A `return` expression
    Return(Option<Box<Expr<'db>>>),
    /// An assignment (`place = expr`)
    Assign(Box<Expr<'db>>, Box<Expr<'db>>, Span<'db>),
    /// An assignment with an operator (`place += expr`)
    AssignOp(AssignOp<'db>, Box<Expr<'db>>, Box<Expr<'db>>),
    /// A unary operation (e.g., `*x`, `-x`, `!x`)
    Unary(UnOp, Box<Expr<'db>>),
    /// A struct literal expression (e.g., `Point { x: 5, y: 10 }`)
    Struct(Box<StructExpr<'db>>),
    /// Field access (e.g., `p.x`)
    Field(Box<Expr<'db>>, Ident<'db>),
    /// Error placeholder
    Err,
}

/// A struct literal expression (e.g., `Point { x: 5, y: 10 }`).
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct StructExpr<'db> {
    pub path: Path<'db>,
    pub fields: ThinVec<ExprField<'db>>,
}

/// A single field initializer in a struct literal expression.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, salsa::Update, serde::Serialize, serde::Deserialize,
)]
pub struct ExprField<'db> {
    pub ident: Ident<'db>,
    pub expr: Box<Expr<'db>>,
    pub span: Span<'db>,
}

impl<'db> scrap_shared::pretty_print::PrettyPrint for ExprKind<'db> {
    fn pretty_print_indent(&self, f: &mut dyn std::fmt::Write, indent: usize) -> std::fmt::Result {
        match self {
            ExprKind::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    elem.pretty_print_indent(f, indent)?;
                }
                write!(f, "]")
            }
            ExprKind::Call(callee, args) => {
                callee.pretty_print_indent(f, indent)?;
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    arg.pretty_print_indent(f, indent)?;
                }
                write!(f, ")")
            }
            ExprKind::Binary(op, left, right) => {
                left.pretty_print_indent(f, indent)?;
                write!(f, " ")?;
                op.node.pretty_print(f)?;
                write!(f, " ")?;
                right.pretty_print_indent(f, indent)
            }
            ExprKind::Lit(lit) => lit.pretty_print_indent(f, indent),
            ExprKind::If(cond, then_block, else_expr) => {
                write!(f, "if ")?;
                cond.pretty_print_indent(f, indent)?;
                write!(f, " ")?;
                then_block.pretty_print_indent(f, indent)?;
                if let Some(else_expr) = else_expr {
                    write!(f, " else ")?;
                    else_expr.pretty_print_indent(f, indent)?;
                }
                Ok(())
            }
            ExprKind::Block(block) => block.pretty_print_indent(f, indent),
            ExprKind::Path(path) => path.pretty_print_indent(f, indent),
            ExprKind::Paren(expr) => {
                write!(f, "(")?;
                expr.pretty_print_indent(f, indent)?;
                write!(f, ")")
            }
            ExprKind::Return(expr) => {
                write!(f, "return")?;
                if let Some(expr) = expr {
                    write!(f, " ")?;
                    expr.pretty_print_indent(f, indent)?;
                }
                Ok(())
            }
            ExprKind::Assign(lhs, rhs, _span) => {
                lhs.pretty_print_indent(f, indent)?;
                write!(f, " = ")?;
                rhs.pretty_print_indent(f, indent)
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                lhs.pretty_print_indent(f, indent)?;
                write!(f, " ")?;
                op.node.pretty_print(f)?;
                write!(f, " ")?;
                rhs.pretty_print_indent(f, indent)
            }
            ExprKind::Unary(op, expr) => {
                let op_str = match op {
                    UnOp::Deref => "*",
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
                };
                write!(f, "{}", op_str)?;
                expr.pretty_print_indent(f, indent)
            }
            ExprKind::Struct(struct_expr) => {
                struct_expr.path.pretty_print_indent(f, indent)?;
                write!(f, " {{ ")?;
                for (i, field) in struct_expr.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    field.ident.pretty_print(f)?;
                    write!(f, ": ")?;
                    field.expr.pretty_print_indent(f, indent)?;
                }
                write!(f, " }}")
            }
            ExprKind::Field(base, field_name) => {
                base.pretty_print_indent(f, indent)?;
                write!(f, ".")?;
                field_name.pretty_print(f)
            }
            ExprKind::Err => write!(f, "<error>"),
        }
    }
}
