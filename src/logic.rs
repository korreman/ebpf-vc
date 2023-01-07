//! A language for logic formulas.

use crate::ast::{BinAlu, UnAlu, Imm};

pub enum BinOp {
    And,
    Or,
    Implies,
    Iff,
}

pub enum Formula {
    True,
    False,
    Not(Box<Formula>),
    BinF(BinOp, Box<(Formula, Formula)>),
    Eq(Expr, Expr),
}

pub enum Expr {
    Value(Imm),
    Var(String),
    Unary(UnAlu, Box<(Expr, Expr)>),
    Binary(BinAlu, Box<(Expr, Expr)>),
}
