//! CVC5 input generation from formulas.

use crate::ast::*;

pub struct Conditions(pub Vec<(String, Formula)>);

impl std::fmt::Display for Conditions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "(set-logic UFBV)\n\
            (set-option :produce-unsat-cores true)\n\
            (declare-fun is_buffer ((_ BitVec 64) (_ BitVec 64)) Bool)\n\n",
        )?;

        for (name, goal) in self.0.iter() {
            f.write_fmt(format_args!(
                "(assert (! (forall (\
                    (r0 (_ BitVec 64)) \
                    (r1 (_ BitVec 64)) \
                    (r2 (_ BitVec 64)) \
                    (r3 (_ BitVec 64)) \
                    (r4 (_ BitVec 64)) \
                    (r5 (_ BitVec 64)) \
                    (r6 (_ BitVec 64)) \
                    (r7 (_ BitVec 64)) \
                    (r8 (_ BitVec 64)) \
                    (r9 (_ BitVec 64))\
                ) {goal}) :named vc_{name}))\n\n"
            ))?;
        }
        f.write_str("(check-sat)\n(get-unsat-core)(exit)")?;
        Ok(())
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Val(imm) => f.write_fmt(format_args!("#x{imm:016x}")),
            Expr::Var(ident) => f.write_str(ident),
            Expr::Unary(op, e) => {
                let op_str = match op {
                    UnAlu::Neg => "bvneg",
                    UnAlu::Le => panic!(),
                    UnAlu::Be => panic!(),
                };
                f.write_fmt(format_args!("({op_str} {e})"))
            }
            Expr::Binary(op, es) => {
                let (e1, e2) = &**es;
                let op_str = match op {
                    BinAlu::Mov => return f.write_fmt(format_args!("{e2}")),
                    BinAlu::Add => "bvadd",
                    BinAlu::Sub => "bvsub",
                    BinAlu::Mul => "bvmul",
                    BinAlu::Div => "bvudiv",
                    BinAlu::Mod => "bvurem",
                    BinAlu::And => "bvand",
                    BinAlu::Or => "bvor",
                    BinAlu::Xor => "bvxor",
                    BinAlu::Lsh => "bvshl",
                    BinAlu::Rsh => "bvlshr",
                    BinAlu::Arsh => panic!(),
                };
                f.write_fmt(format_args!("({op_str} {e1} {e2})"))
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
                    FBinOp::AndAsym => {
                        return f.write_fmt(format_args!("(and {f1} (=> {f1} {f2}))"))
                    }
                };
                f.write_fmt(format_args!("({op_str} {f1} {f2})"))
            }
            Formula::Quant(q, id, form) => {
                f.write_fmt(format_args!("({q} (({id} (_ BitVec 64))) {form})"))
            }
            Formula::Rel(rel, e1, e2) => {
                if *rel == Cc::Ne {
                    return Formula::Not(Box::new(Formula::Rel(Cc::Eq, e1.clone(), e2.clone())))
                        .fmt(f);
                }
                let mut flip = false;
                let rel_str = match rel {
                    Cc::Eq => "=",
                    Cc::Ne => panic!(),
                    Cc::Gt => {
                        flip = true;
                        "bvult"
                    }
                    Cc::Ge => {
                        flip = true;
                        "bvule"
                    }
                    Cc::Lt => "bvult",
                    Cc::Le => "bvule",
                    Cc::Set => todo!(),
                    Cc::Sgt => todo!(),
                    Cc::Sge => todo!(),
                    Cc::Slt => todo!(),
                    Cc::Sle => todo!(),
                };
                if flip {
                    f.write_fmt(format_args!("({rel_str} {e2} {e1})"))
                } else {
                    f.write_fmt(format_args!("({rel_str} {e1} {e2})"))
                }
            }
            Formula::IsBuffer(ptr, sz) => f.write_fmt(format_args!("(is_buffer {ptr} {sz})")),
        }
    }
}
