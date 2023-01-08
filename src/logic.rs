//! A language for logic formulas.
use std::collections::HashMap;

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

pub struct FormulaBuilder {
    id_counters: HashMap<String, usize>,
}

impl FormulaBuilder {
    pub fn new() -> Self {
        Self { id_counters: HashMap::new() }
    }

    pub fn top(&self) -> Formula {
        Formula::Val(true)
    }

    pub fn bot(&self) -> Formula {
        Formula::Val(false)
    }

    /// Generate a new, unique variable.
    pub fn var(&mut self, mut ident: Ident) -> Expr {
        let counter = self.id_counters.entry(ident.clone()).or_insert(0);
        ident.push('_');
        ident.extend(format!("{counter}").chars());
        *counter += 1;
        Expr::Var(ident)
    }

    pub fn not(&self, f: Formula) -> Formula {
        Formula::Not(Box::new(f))
    }

    pub fn and(&self, a: Formula, b: Formula) -> Formula {
        Formula::Bin(BinOp::And, Box::new((a, b)))
    }

    pub fn or(&self, a: Formula, b: Formula) -> Formula {
        Formula::Bin(BinOp::Or, Box::new((a, b)))
    }

    pub fn implies(&self, a: Formula, b: Formula) -> Formula {
        Formula::Bin(BinOp::Implies, Box::new((a, b)))
    }

    pub fn iff(&self, a: Formula, b: Formula) -> Formula {
        Formula::Bin(BinOp::Iff, Box::new((a, b)))
    }

    pub fn forall(&self, ident: Ident, f: Formula) -> Formula {
        Formula::Quant(QType::Forall, ident, Box::new(f))
    }

    pub fn exists(&self, ident: Ident, f: Formula) -> Formula {
        Formula::Quant(QType::Exists, ident, Box::new(f))
    }

    pub fn replace(&self, prev: Ident, new: Ident, f: Formula) -> Formula {
        Formula::Replace {
            prev,
            new,
            f: Box::new(f),
        }
    }

    pub fn rel(&self, cc: Cc, a: Expr, b: Expr) -> Formula {
        Formula::Rel(cc, a, b)
    }

    pub fn eq(&self, a: Expr, b: Expr) -> Formula {
        Formula::Rel(Cc::Eq, a, b)
    }

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

pub fn e_unop(op: UnAlu, e: Expr) -> Expr {
    Expr::Unary(op, Box::new(e))
}

pub fn e_binop(op: BinAlu, a: Expr, b: Expr) -> Expr {
    Expr::Binary(op, Box::new((a, b)))
}
