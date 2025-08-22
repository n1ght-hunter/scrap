use std::{collections::HashMap, marker::PhantomData};

use crate::Span;

#[derive(Debug, Clone)]
pub struct Item<'a> {
    pub kind: ItemKind<'a>,
    pub span: Span,
    // pub id: NodeId,
    // pub vis: Visibility,
}

#[derive(Debug, Clone)]
pub enum ItemKind<'a> {
    Error,
    Fn(FnDef<'a>),
    Enum(EnumDef<'a>),
    Struct(StructDef<'a>),
}

#[derive(Debug, Clone, Copy)]
pub struct Symbol(pub u32);
#[derive(Debug, Clone, Copy)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnDef<'a> {
    pub _p: PhantomData<&'a str>,
    pub ident: Ident,
    pub args: Vec<(Ident, Type)>,
    pub ret_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct Type(pub Ident);

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Unit(Ident),
    Full { name: Ident, ty: Type },
}

#[derive(Debug, Clone)]
pub struct EnumDef<'a> {
    pub _p: PhantomData<&'a str>,
    pub ident: Ident,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct StructDef<'a> {
    pub _p: PhantomData<&'a str>,
    pub ident: Ident,
    pub fields: Vec<(Ident, Type)>,
}

#[derive(Debug, Clone)]
pub enum FileExpr<'a> {
    Function {
        name: &'a str,
        args: Vec<&'a str>,
        body: Expr<'a>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Literal(Literal<'a>),
    Variable(&'a str),
    BinaryOp {
        op: char,
        left: Box<Expr<'a>>,
        right: Box<Expr<'a>>,
    },
    If {
        condition: Box<Expr<'a>>,
        then_block: Vec<Stmt<'a>>,
        else_block: Option<Vec<Stmt<'a>>>,
    },
}

#[derive(Debug, Clone)]
pub enum Literal<'a> {
    Int(&'a str),
    Float(&'a str),
    Str(&'a str),
}

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    Let { var_name: &'a str, value: Expr<'a> },
    Expr(Expr<'a>),
    // A semicolon statement
    Empty,
}

#[derive(Debug, Clone)]
pub struct Field<'a> {
    pub name: &'a str,
    pub field_type: &'a str,
}
