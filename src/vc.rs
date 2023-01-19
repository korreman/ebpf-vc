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

#[derive(Debug, PartialEq, Eq)]
enum BlockStatus {
    Pending,
    PreCond(Formula),
    Cyclic,
}

pub fn vc(module: Module) -> Result<Vec<Formula>, VcError> {
    // Stores results.
    let mut verif_conds: Vec<Formula> = Vec::new();

    // Stores cached pre-conditions for each block.
    // Also used to track which blocks have already been visited.
    let mut pre_conds: HashMap<Label, BlockStatus> = HashMap::new();

    // Perform a reverse breadth-first traversal of CFG.
    let mut stack = vec![module.start.clone()];
    let mut f = FormulaBuilder::new();
    while let Some(label) = stack.pop() {
        // Try to mark block as pending.
        let status = pre_conds
            .entry(label.clone())
            .or_insert(BlockStatus::Pending);
        // Skip block if it has already been processed.
        if matches!(*status, BlockStatus::PreCond(_)) {
            continue;
        }
        let block = &module.blocks[&label];

        let mut get_post_cond = |target: &Label| {
            let b = pre_conds.get_mut(target);
            match b {
                // If already processed, return the result.
                Some(BlockStatus::PreCond(c)) => Some(c.clone()),
                // If pending, use the pre-assertion if it exists and fail if it doesn't.
                Some(BlockStatus::Pending) | Some(BlockStatus::Cyclic) => {
                    // Mark block as cyclic.
                    *b.unwrap() = BlockStatus::Cyclic;
                    if let Some(c) = &module.blocks[target].invariant {
                        Some(c.clone())
                    } else {
                        Some(f.top())
                    }
                }
                // If block isn't marked as anything, push it to the stack.
                None => {
                    stack.push(label.clone());
                    stack.push(target.clone());
                    None
                }
            }
        };

        // Generate post-condition from continuation.
        let post_cond = match &block.next {
            Continuation::Exit => Some(f.top()),
            Continuation::Jmp(target) => get_post_cond(target),
            Continuation::Jcc(cc, lhs, rhs, target_t, target_f) => {
                // First, get post-condition of the two targets.
                let cond_t = get_post_cond(target_t);
                let cond_f = get_post_cond(target_f);

                // Next, build formula for comparison.
                let lhs = f.reg(*lhs).0;
                let rhs = match rhs {
                    RegImm::Reg(r) => f.reg(*r).0,
                    RegImm::Imm(i) => f.val(*i),
                };
                let cc = f.rel(*cc, lhs, rhs);

                // Generate condition as conjugation between the two branches.
                cond_t.zip(cond_f).map(|(cond_t, cond_f)| {
                    f.or(
                        f.asym_and(cc.clone(), cond_t.clone()),
                        f.asym_and(f.not(cc), cond_f.clone()),
                    )
                })
            }
        };
        let post_cond = match post_cond {
            Some(c) => c,
            None => continue,
        };

        // Perform WP-calculus on post-condition with block body.
        let wp_result = wp(&mut f, &block.body, post_cond);

        // Cache or use result of WP.
        let top = f.top();
        let pre_assert = block.invariant.as_ref();
        let pre_assert = pre_assert.or(if pre_conds[&label] == BlockStatus::Cyclic {
            Some(&top)
        } else {
            None
        });

        if let Some(pre_assert) = pre_assert {
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
    verif_conds.push(match &pre_conds[&module.start] {
        BlockStatus::PreCond(c) => c.clone(),
        _ => panic!("starting block is never processed"),
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

                // Add extra conditions for division/modulo by zero.
                if op == &BinAlu::Div || op == &BinAlu::Mod {
                    cond = f.asym_and(f.rel(Cc::Ne, s, f.val(0)), cond);
                }
            }
            instr => panic!("not implemented: {instr:?}"),
        }
    }
    cond
}
