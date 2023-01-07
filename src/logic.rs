//! A language for logic formulas.
use crate::ast::{BinAlu, Cc, Imm, Reg, UnAlu};

pub type Ident = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    And,
    Or,
    Implies,
    Iff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QType {
    Exists,
    Forall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Formula {
    Val(bool),
    Not(Box<Formula>),
    Bin(BinOp, Box<(Formula, Formula)>),
    Quant(QType, Ident, Box<Formula>),
    Replace {
        prev: Ident,
        new: Ident,
        f: Box<Formula>,
    },
    Rel(Cc, Expr, Expr),
}

pub fn reg_to_ident(reg: Reg) -> Ident {
    format!("r{}", reg.get())
}

pub fn reg_to_var(reg: Reg) -> Expr {
    Expr::Var(format!("r{}", reg.get()))
}

pub fn f_true() -> Formula {
    Formula::Val(true)
}

pub fn f_false() -> Formula {
    Formula::Val(false)
}

pub fn f_not(f: Formula) -> Formula {
    Formula::Not(Box::new(f))
}

pub fn f_and(a: Formula, b: Formula) -> Formula {
    Formula::Bin(BinOp::And, Box::new((a, b)))
}

pub fn f_or(a: Formula, b: Formula) -> Formula {
    Formula::Bin(BinOp::Or, Box::new((a, b)))
}

pub fn f_implies(a: Formula, b: Formula) -> Formula {
    Formula::Bin(BinOp::Implies, Box::new((a, b)))
}

pub fn f_iff(a: Formula, b: Formula) -> Formula {
    Formula::Bin(BinOp::Iff, Box::new((a, b)))
}

pub fn f_forall(ident: Ident, f: Formula) -> Formula {
    Formula::Quant(QType::Forall, ident, Box::new(f))
}

pub fn f_exists(ident: Ident, f: Formula) -> Formula {
    Formula::Quant(QType::Exists, ident, Box::new(f))
}

pub fn f_replace(prev: Ident, new: Ident, f: Formula) -> Formula {
    Formula::Replace {
        prev,
        new,
        f: Box::new(f),
    }
}

pub fn f_rel(cc: Cc, a: Expr, b: Expr) -> Formula {
    Formula::Rel(cc, a, b)
}

pub fn f_eq(a: Expr, b: Expr) -> Formula {
    Formula::Rel(Cc::Eq, a, b)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Val(Imm),
    Var(Ident),
    Unary(UnAlu, Box<Expr>),
    Binary(BinAlu, Box<(Expr, Expr)>),
}

pub fn e_val(i: Imm) -> Expr {
    Expr::Val(i)
}

pub fn e_var(ident: &str) -> Expr {
    Expr::Var(String::from(ident))
}

pub fn e_unop(op: UnAlu, e: Expr) -> Expr {
    Expr::Unary(op, Box::new(e))
}

pub fn e_binop(op: BinAlu, a: Expr, b: Expr) -> Expr {
    Expr::Binary(op, Box::new((a, b)))
}
