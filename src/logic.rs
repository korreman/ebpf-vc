//! A language for logic formulas.

use crate::ast::{BinAlu, Imm, Reg, RegImm, UnAlu};

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
    Eq(Expr, Expr),
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

pub fn f_eq(a: Expr, b: Expr) -> Formula {
    Formula::Eq(a, b)
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

pub fn e_var(ident: Ident) -> Expr {
    Expr::Var(ident)
}

pub fn e_unop(op: UnAlu, e: Expr) -> Expr {
    Expr::Unary(op, Box::new(e))
}

pub fn e_binop(op: BinAlu, dst: Reg, src: RegImm) -> Expr {
    let lhs = reg_to_var(dst);
    let rhs = match src {
        RegImm::Reg(r) => reg_to_var(r),
        RegImm::Imm(i) => Expr::Val(i),
    };
    Expr::Binary(op, Box::new((lhs, rhs)))
}

pub fn e_mov(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Mov, dst, src)
}
pub fn e_add(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Add, dst, src)
}
pub fn e_sub(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Sub, dst, src)
}
pub fn e_mul(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Mul, dst, src)
}
pub fn e_div(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Div, dst, src)
}
pub fn e_mod(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Mod, dst, src)
}
pub fn e_and(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::And, dst, src)
}
pub fn e_or(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Or, dst, src)
}
pub fn e_xor(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Xor, dst, src)
}
pub fn e_lsh(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Lsh, dst, src)
}
pub fn e_rsh(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Rsh, dst, src)
}
pub fn e_arsh(dst: Reg, src: RegImm) -> Expr {
    e_binop(BinAlu::Arsh, dst, src)
}
