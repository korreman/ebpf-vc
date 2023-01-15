//! Verification condition generation.
mod preprocess;
pub use preprocess::*;

pub mod ast;
use crate::{ast::WordSize, logic::*};
use ast::*;

enum BlockStatus {
    Untouched,
    Pending,
    PreCond(Formula),
}

pub fn vc(module: Module) -> Option<Vec<Formula>> {
    // Stores results.
    let mut verif_conds: Vec<Formula> = Vec::new();

    // Stores cached pre-conditions for each block.
    // Also used to track which blocks have already been visited.
    let mut pre_conds: Vec<_> = module
        .blocks
        .iter()
        .map(|_| BlockStatus::Untouched)
        .collect();

    // Perform a reverse breadth-first traversal of CFG.
    let mut stack = vec![0];
    let mut f = FormulaBuilder::new();
    while let Some(label) = stack.pop() {
        let block = &module.blocks[label];
        pre_conds[label] = BlockStatus::Pending;

        // Generate post-condition from continuation.
        // TODO: Generalize retrieval of post-conditions.
        let post_cond = match block.next {
            Continuation::Exit => f.top(),
            Continuation::Jmp(target) => match &pre_conds[target] {
                BlockStatus::PreCond(c) => c.clone(),
                BlockStatus::Untouched => {
                    stack.push(label);
                    stack.push(target);
                    continue;
                }
                BlockStatus::Pending => {
                    if let Some(c) = &module.blocks[target].pre_assert {
                        c.clone()
                    } else {
                        return None;
                    }
                }
            },
            Continuation::Jcc(cc, lhs, rhs, target_t, target_f) => {
                let cond_t = match &pre_conds[target_t] {
                    BlockStatus::PreCond(c) => c.clone(),
                    BlockStatus::Untouched => {
                        stack.push(label);
                        stack.push(target_t);
                        continue;
                    }
                    BlockStatus::Pending => {
                        if let Some(c) = &module.blocks[target_t].pre_assert {
                            c.clone()
                        } else {
                            return None;
                        }
                    }
                };
                let cond_f = match &pre_conds[target_f] {
                    BlockStatus::PreCond(c) => c.clone(),
                    BlockStatus::Untouched => {
                        stack.push(label);
                        stack.push(target_f);
                        continue;
                    }
                    BlockStatus::Pending => {
                        if let Some(c) = &module.blocks[target_f].pre_assert {
                            c.clone()
                        } else {
                            return None;
                        }
                    }
                };
                let lhs = f.reg(lhs).0;
                let rhs = match rhs {
                    RegImm::Reg(r) => f.reg(r).0,
                    RegImm::Imm(i) => f.val(i),
                };
                let cc = f.rel(cc, lhs, rhs);
                f.or(
                    f.and(cc.clone(), cond_t.clone()),
                    f.and(f.not(cc), cond_f.clone()),
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
            pre_conds[label] = BlockStatus::PreCond(pre_assert.clone());
        } else {
            // Otherwise, cache the WP result for the block.
            pre_conds[label] = BlockStatus::PreCond(wp_result);
        }
    }

    // Add the pre-condition of the starting block as a VC.
    verif_conds.push(match &pre_conds[0] {
        BlockStatus::PreCond(c) => c.clone(),
        _ => panic!("starting block never resolved"),
    });
    Some(verif_conds)
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
