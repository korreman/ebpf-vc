//! Verification condition generation.
mod preprocess;
use std::{collections::HashMap, fmt::Display};

pub use preprocess::*;

pub mod ast;
use crate::{
    ast::{Formula, Label, WordSize},
    logic::*,
};
use ast::*;

pub enum VcError {
    NoPreAssert(Label),
}

impl Display for VcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VcError::NoPreAssert(label) => {
                f.write_fmt(format_args!("missing pre-assertion for {label:?}"))
            }
        }
    }
}

enum BlockStatus {
    Pending,
    PreCond(Formula),
}

pub fn vc(module: Module) -> Result<Vec<Formula>, VcError> {
    // Stores results.
    let mut verif_conds: Vec<Formula> = Vec::new();

    // Stores cached pre-conditions for each block.
    // Also used to track which blocks have already been visited.
    let mut pre_conds: HashMap<Label, BlockStatus> = HashMap::new();

    // Perform a reverse breadth-first traversal of CFG.
    let mut stack = vec![module.start];
    let mut f = FormulaBuilder::new();
    while let Some(label) = stack.pop() {
        drop(
            pre_conds
                .entry(label.clone())
                .or_insert(BlockStatus::Pending),
        );
        let block = &module.blocks[&label];

        // Generate post-condition from continuation.
        // TODO: Generalize retrieval of post-conditions.
        let post_cond = match &block.next {
            Continuation::Exit => f.top(),
            Continuation::Jmp(target) => match &pre_conds.get(target) {
                Some(BlockStatus::PreCond(c)) => c.clone(),
                Some(BlockStatus::Pending) => {
                    if let Some(c) = &module.blocks[target].pre_assert {
                        c.clone()
                    } else {
                        return Err(VcError::NoPreAssert(target.clone()));
                    }
                }
                None => {
                    stack.push(label);
                    stack.push(target.clone());
                    continue;
                }
            },
            Continuation::Jcc(cc, lhs, rhs, target_t, target_f) => {
                let cond_t = match &pre_conds.get(target_t) {
                    Some(BlockStatus::PreCond(c)) => c.clone(),
                    Some(BlockStatus::Pending) => {
                        if let Some(c) = &module.blocks[target_t].pre_assert {
                            c.clone()
                        } else {
                            return Err(VcError::NoPreAssert(target_t.clone()));
                        }
                    }
                    None => {
                        stack.push(label);
                        stack.push(target_t.clone());
                        continue;
                    }
                };
                let cond_f = match &pre_conds.get(target_f) {
                    Some(BlockStatus::PreCond(c)) => c.clone(),
                    Some(BlockStatus::Pending) => {
                        if let Some(c) = &module.blocks[target_f].pre_assert {
                            c.clone()
                        } else {
                            return Err(VcError::NoPreAssert(target_f.clone()));
                        }
                    }
                    None => {
                        stack.push(label);
                        stack.push(target_f.clone());
                        continue;
                    }
                };
                let lhs = f.reg(*lhs).0;
                let rhs = match rhs {
                    RegImm::Reg(r) => f.reg(*r).0,
                    RegImm::Imm(i) => f.val(*i),
                };
                let cc = f.rel(*cc, lhs, rhs);
                f.or(
                    f.asym_and(cc.clone(), cond_t.clone()),
                    f.asym_and(f.not(cc), cond_f.clone()),
                )
            }
        };

        // Perform WP-calculus on post-condition with block body.
        let wp_result = wp(&mut f, &block.body, post_cond);

        // Cache or use result of WP.
        if let Some(pre_assert) = &block.pre_assert {
            // If the block has a pre-assertion,
            // add a VC requiring that the pre-assertion implies the WP result.
            verif_conds.push(f.implies(pre_assert.clone(), wp_result));
            pre_conds.insert(label, BlockStatus::PreCond(pre_assert.clone()));
        } else {
            // Otherwise, cache the WP result for the block.
            pre_conds.insert(label, BlockStatus::PreCond(wp_result));
        }
    }

    // Add the pre-condition of the starting block as a VC.
    verif_conds.push(match &pre_conds["@0"] {
        BlockStatus::PreCond(c) => c.clone(),
        _ => panic!("starting block never resolved"),
    });
    Ok(verif_conds)
}

fn wp(f: &mut FormulaBuilder, instrs: &[Instr], mut cond: Formula) -> Formula {
    for instr in instrs.iter().rev() {
        match instr {
            Instr::Unary(WordSize::B64, op, reg) => {
                let (t, t_id) = f.reg(*reg);
                let e = f.unop(*op, t);

                let (v, v_id) = f.var(String::from("v"));
                cond = f.forall(
                    v_id.clone(),
                    f.implies(f.eq(v, e), f.replace(&t_id, &v_id, cond)),
                );
            }
            Instr::Binary(WordSize::B64, op, dst, src) => {
                let (d, d_id) = f.reg(*dst);
                let s = match src {
                    RegImm::Reg(r) => f.reg(*r).0,
                    RegImm::Imm(i) => f.val(*i),
                };
                let e = f.binop(*op, d, s.clone());
                let (v, v_id) = f.var(String::from("v"));
                cond = f.forall(
                    v_id.clone(),
                    f.implies(f.eq(v, e), f.replace(&d_id, &v_id, cond)),
                );

                // Handle division/modulo by zero.
                if op == &BinAlu::Div || op == &BinAlu::Mod {
                    cond = f.asym_and(f.rel(Cc::Ne, s, f.val(0)), cond);
                }
            }
            instr => panic!("not implemented: {instr:?}"),
        }
    }
    cond
}
