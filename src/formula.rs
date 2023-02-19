//! A stateful builder for formulas.

use std::collections::HashMap;

use crate::ast::*;

#[derive(Default)]
pub struct FormulaBuilder {
    id_counters: HashMap<String, usize>,
}

impl FormulaBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn top(&self) -> Formula {
        Formula::Val(true)
    }

    pub fn bot(&self) -> Formula {
        Formula::Val(false)
    }

    pub fn not(&self, f: Formula) -> Formula {
        Formula::Not(Box::new(f))
    }

    pub fn and(&self, a: Formula, b: Formula) -> Formula {
        Formula::Bin(FBinOp::And, Box::new((a, b)))
    }

    pub fn asym_and(&self, a: Formula, b: Formula) -> Formula {
        Formula::Bin(FBinOp::AndAsym, Box::new((a, b)))
    }

    pub fn or(&self, a: Formula, b: Formula) -> Formula {
        Formula::Bin(FBinOp::Or, Box::new((a, b)))
    }

    pub fn implies(&self, a: Formula, b: Formula) -> Formula {
        Formula::Bin(FBinOp::Implies, Box::new((a, b)))
    }

    pub fn iff(&self, a: Formula, b: Formula) -> Formula {
        Formula::Bin(FBinOp::Iff, Box::new((a, b)))
    }

    pub fn forall(&self, ident: Ident, f: Formula) -> Formula {
        Formula::Quant(QType::Forall, ident, Box::new(f))
    }

    pub fn exists(&self, ident: Ident, f: Formula) -> Formula {
        Formula::Quant(QType::Exists, ident, Box::new(f))
    }

    pub fn replace(&self, prev: &Ident, new: &Ident, f: &Formula) -> Option<Formula> {
        match f {
            Formula::Not(inner) => {
                let res = self.replace(prev, new, inner)?;
                Some(Formula::Not(Box::new(res)))
            }
            Formula::Bin(op, fs) => {
                let a = self.replace(prev, new, &fs.0);
                let b = self.replace(prev, new, &fs.1);
                if a.is_some() || b.is_some() {
                    Some(Formula::Bin(
                        *op,
                        Box::new((a.unwrap_or(fs.0.clone()), b.unwrap_or(fs.1.clone()))),
                    ))
                } else {
                    None
                }
            }
            Formula::Quant(qtype, qvar, inner) => {
                if prev != qvar {
                    let res = self.replace(prev, new, inner)?;
                    Some(Formula::Quant(*qtype, qvar.clone(), Box::new(res)))
                } else {
                    None
                }
            }
            Formula::Rel(r, e1, e2) => {
                let a = self.replace_expr(prev, new, e1);
                let b = self.replace_expr(prev, new, e2);
                if a.is_some() || b.is_some() {
                    Some(Formula::Rel(
                        *r,
                        a.unwrap_or(e1.clone()),
                        b.unwrap_or(e2.clone()),
                    ))
                } else {
                    None
                }
            }
            Formula::IsBuffer(ptr, sz) => {
                let new_ptr = if ptr == prev { Some(new.clone()) } else { None };
                let new_sz = self.replace_expr(prev, new, sz);
                if new_ptr.is_some() || new_sz.is_some() {
                    Some(Formula::IsBuffer(
                        new_ptr.unwrap_or(ptr.clone()),
                        new_sz.unwrap_or(sz.clone()),
                    ))
                } else {
                    None
                }
            }
            Formula::Val(_) => None,
        }
    }

    pub fn replace_expr(&self, prev: &Ident, new: &Ident, e: &Expr) -> Option<Expr> {
        match e {
            Expr::Var(x) => {
                if x == prev {
                    Some(Expr::Var(new.clone()))
                } else {
                    None
                }
            }
            Expr::Unary(op, inner) => {
                let res = self.replace_expr(prev, new, inner)?;
                Some(Expr::Unary(*op, Box::new(res)))
            }
            Expr::Binary(op, es) => {
                let a = self.replace_expr(prev, new, &es.0);
                let b = self.replace_expr(prev, new, &es.1);
                if a.is_some() || b.is_some() {
                    Some(Expr::Binary(
                        *op,
                        Box::new((a.unwrap_or(es.0.clone()), b.unwrap_or(es.1.clone()))),
                    ))
                } else {
                    None
                }
            }
            Expr::Val(_) => None,
        }
    }

    pub fn rel(&self, cc: Cc, a: Expr, b: Expr) -> Formula {
        Formula::Rel(cc, a, b)
    }

    pub fn eq(&self, a: Expr, b: Expr) -> Formula {
        Formula::Rel(Cc::Eq, a, b)
    }

    /// Generate a new, unique variable.
    pub fn var(&mut self, mut ident: Ident) -> (Expr, Ident) {
        let counter = self.id_counters.entry(ident.clone()).or_insert(0);
        ident.extend(format!("{counter}").chars());
        *counter += 1;
        (Expr::Var(ident.clone()), ident)
    }

    /// Generate a non-unique expression representing [ident].
    pub fn var_ident(&self, ident: Ident) -> Expr {
        Expr::Var(ident)
    }

    /// Get the variable representing a register.
    pub fn reg(&self, reg: Reg) -> (Expr, Ident) {
        let id = format!("r{}", reg.get());
        (Expr::Var(id.clone()), id)
    }

    pub fn val(&self, i: Imm) -> Expr {
        Expr::Val(i)
    }

    pub fn unop(&self, op: UnAlu, e: Expr) -> Expr {
        Expr::Unary(op, Box::new(e))
    }

    pub fn binop(&self, op: BinAlu, a: Expr, b: Expr) -> Expr {
        Expr::Binary(op, Box::new((a, b)))
    }

    pub fn is_buffer(&self, ptr: Ident, size: Expr) -> Formula {
        Formula::IsBuffer(ptr, size)
    }
}
