//! Verification condition generation.
use std::mem::swap;

mod preprocess;
pub use preprocess::*;

pub mod ast;
use crate::{ast::WordSize, logic::*};
use ast::*;

pub fn vc(module: Module) -> Option<Vec<Formula>> {
    // Stores results.
    let mut verif_conds: Vec<Formula> = Vec::new();

    // Stores cached pre-conditions for each block.
    // Also used to track which blocks have already been visited.
    let mut pre_conds: Vec<Option<Formula>> = module.blocks.iter().map(|_| None).collect();

    // Create copy of the CFG with the edges reversed.
    // Pre-emptively cache the pre-assertions as pre-conditions.
    let mut reverse_graph: Vec<Vec<Label>> = module.blocks.iter().map(|_| Vec::new()).collect();
    for (label, block) in module.blocks.iter().enumerate() {
        match block.next {
            Continuation::Jmp(target) => reverse_graph[target].push(label),
            Continuation::Jcc(_, _, _, target_t, target_f) => {
                reverse_graph[target_t].push(label);
                reverse_graph[target_f].push(label);
            }
            Continuation::Exit => {}
        }
        if let Some(pre_assert) = &block.pre_assert {
            pre_conds[label] = Some(pre_assert.clone());
        }
    }

    // Collect all blocks that end with an exit.
    let mut labels: Vec<Label> = module
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block.next == Continuation::Exit)
        .map(|(label, _)| label)
        .collect();

    // Perform a reverse breadth-first traversal of CFG.
    let mut f = FormulaBuilder::new();
    let mut next_labels: Vec<usize> = Vec::new();
    while !labels.is_empty() {
        for label in labels.drain(..) {
            let block = &module.blocks[label];
            // Generate post-condition from continuation.
            let post_cond = match block.next {
                Continuation::Jcc(cc, lhs, rhs, target_t, target_f) => {
                    let lhs = f.reg(lhs).0;
                    let rhs = match rhs {
                        RegImm::Reg(r) => f.reg(r).0,
                        RegImm::Imm(i) => f.val(i),
                    };
                    let cond = f.rel(cc, lhs, rhs);
                    if let (Some(cond_t), Some(cond_f)) =
                        (&pre_conds[target_t], &pre_conds[target_f])
                    {
                        f.or(
                            f.and(cond.clone(), cond_t.clone()),
                            f.and(f.not(cond), cond_f.clone()),
                        )
                    } else {
                        continue;
                    }
                }
                Continuation::Jmp(target) => pre_conds[target]
                    .clone()
                    .expect("block {target} should already have a pre-condition"),
                Continuation::Exit => f.top(),
            };
            // Perform WP-calculus on post-condition with block body.
            let wp_result = wp(&mut f, &block.body, post_cond);
            // Cache or use result of WP.
            if let Some(pre_assert) = &block.pre_assert {
                // If the block has a pre-assertion,
                // add a VC requiring that the pre-assertion implies the WP result.
                let extra_cond = f.implies(pre_assert.clone(), wp_result);
                verif_conds.push(extra_cond);
            } else {
                // If the block doesn't have a pre-assertion,
                // but has already been visited,
                // abort.
                if pre_conds[label].is_some() {
                    return None;
                }
                // Otherwise, cache the WP result for the block.
                pre_conds[label] = Some(wp_result);
            }
            // Add blocks that target this one to the next round.
            next_labels.extend(reverse_graph[label].iter());
        }
        // Prepare labels for next round.
        next_labels.sort();
        next_labels.dedup();
        swap(&mut labels, &mut next_labels);
    }
    // Add the pre-condition of the starting block as a VC.
    verif_conds.push(
        pre_conds
            .into_iter()
            .next()
            .expect("empty input")
            .expect("didn't reach start"),
    );
    Some(verif_conds)
}

fn wp(f: &mut FormulaBuilder, instrs: &[Instr], mut cond: Formula) -> Formula {
    for instr in instrs.iter().rev() {
        match instr {
            Instr::Unary(WordSize::B64, op, reg) => {
                let (t, t_id) = f.reg(*reg);
                let (v, v_id) = f.var(String::from("v"));
                let e = f.unop(*op, t);
                cond = f.forall(
                    v_id.clone(),
                    f.implies(f.eq(v, e), f.replace(t_id, v_id, cond)),
                )
            }
            Instr::Binary(WordSize::B64, op, dst, src) => {
                let (v, v_id) = f.var(String::from("v"));
                let (d, d_id) = f.reg(*dst);
                let s = match src {
                    RegImm::Reg(r) => f.reg(*r).0,
                    RegImm::Imm(i) => f.val(*i),
                };
                let e = f.binop(*op, d, s);
                cond = f.forall(
                    v_id.clone(),
                    f.implies(f.eq(v, e), f.replace(d_id, v_id, cond)),
                )
            }
            instr => panic!("not implemented: {instr:?}"),
        }
    }
    cond
}
