//! A language for logic formulas.
use std::collections::HashMap;

use crate::ast::*;

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Val(imm) => f.write_fmt(format_args!("{imm}")),
            Expr::Var(ident) => f.write_str(ident),
            Expr::Unary(op, e) => {
                let op_str = match op {
                    UnAlu::Neg => "neg",
                    UnAlu::Le => "le",
                    UnAlu::Be => "be",
                };
                f.write_fmt(format_args!("{op_str}({e})"))
            }
            Expr::Binary(op, es) => {
                let (e1, e2) = &**es;
                let op_str = match op {
                    BinAlu::Mov => return f.write_fmt(format_args!("{e2}")),
                    BinAlu::Add => "+",
                    BinAlu::Sub => "-",
                    BinAlu::Mul => "*",
                    BinAlu::Div => return f.write_fmt(format_args!("div {e1} {e2}")),
                    BinAlu::Mod => return f.write_fmt(format_args!("mod {e1} {e2}")),
                    BinAlu::And => "&",
                    BinAlu::Or => "|",
                    BinAlu::Xor => "^",
                    BinAlu::Lsh => "<<",
                    BinAlu::Rsh => ">>",
                    BinAlu::Arsh => ">>>",
                };
                f.write_fmt(format_args!("({e1} {op_str} {e2})"))
            }
        }
    }
}

impl std::fmt::Display for QType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            QType::Exists => "exists",
            QType::Forall => "forall",
        })
    }
}

impl std::fmt::Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formula::Val(b) => f.write_str(if *b { "true" } else { "false" }),
            Formula::Not(form) => f.write_fmt(format_args!("not ({form})")),
            Formula::Bin(op, fs) => {
                let (f1, f2) = &**fs;
                let op_str = match op {
                    FBinOp::And => "/\\",
                    FBinOp::Or => "\\/",
                    FBinOp::Implies => "->",
                    FBinOp::Iff => "<->",
                    FBinOp::AndAsym => "&&",
                };
                f.write_fmt(format_args!("({f1} {op_str} {f2})"))
            }
            Formula::Quant(q, id, form) => {
                f.write_fmt(format_args!("({q} {id} : uint64 . {form})"))
            }
            Formula::Rel(rel, e1, e2) => {
                let rel_str = match rel {
                    Cc::Eq => "=",
                    Cc::Gt => ">",
                    Cc::Ge => ">=",
                    Cc::Lt => "<",
                    Cc::Le => "<=",
                    Cc::Set => todo!(),
                    Cc::Ne => "<>",
                    Cc::Sgt => todo!(),
                    Cc::Sge => todo!(),
                    Cc::Slt => todo!(),
                    Cc::Sle => todo!(),
                };
                f.write_fmt(format_args!("({e1} {rel_str} {e2})"))
            }
            Formula::IsBuffer(ptr, sz) => f.write_fmt(format_args!("is_buffer {ptr} {sz}")),
        }
    }
}

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

    pub fn replace(&self, prev: &Ident, new: &Ident, mut f: Formula) -> Formula {
        match &mut f {
            Formula::Not(inner) => **inner = self.replace(prev, new, *inner.clone()),
            Formula::Bin(_, fs) => {
                fs.0 = self.replace(prev, new, fs.0.clone());
                fs.1 = self.replace(prev, new, fs.1.clone());
            }
            Formula::Quant(_, qvar, inner) => {
                if prev != qvar {
                    **inner = self.replace(prev, new, *inner.clone());
                }
            }
            Formula::Rel(_, e1, e2) => {
                *e1 = self.replace_expr(prev, new, e1.clone());
                *e2 = self.replace_expr(prev, new, e2.clone());
            }
            Formula::IsBuffer(ptr, sz) => {
                if ptr == prev {
                    *ptr = new.clone();
                }
                *sz = self.replace_expr(prev, new, sz.clone());
            }
            Formula::Val(_) => (),
        }
        f
    }

    pub fn replace_expr(&self, prev: &Ident, new: &Ident, mut e: Expr) -> Expr {
        match &mut e {
            Expr::Var(x) => {
                if x == prev {
                    *x = new.clone();
                }
            }
            Expr::Unary(_, inner) => {
                **inner = self.replace_expr(prev, new, *inner.clone());
            }
            Expr::Binary(_, es) => {
                es.0 = self.replace_expr(prev, new, es.0.clone());
                es.1 = self.replace_expr(prev, new, es.1.clone());
            }
            Expr::Val(_) => (),
        }
        e
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
