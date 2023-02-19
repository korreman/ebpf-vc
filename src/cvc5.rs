//! CVC5 input generation from formulas.

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
            Formula::Not(form) => f.write_fmt(format_args!("(not {form})")),
            Formula::Bin(op, fs) => {
                let (f1, f2) = &**fs;
                let op_str = match op {
                    FBinOp::And => "and",
                    FBinOp::Or => "or",
                    FBinOp::Implies => "=>",
                    FBinOp::Iff => "<=>",
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
