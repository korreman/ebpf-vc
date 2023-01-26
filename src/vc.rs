//! Verification condition generation.

use std::collections::HashMap;

use crate::{cfg::*, formula::*};

#[derive(Debug, PartialEq, Eq)]
enum BlockStatus {
    Pending,
    Cyclic,
    PreCond(Formula),
}

pub fn vc(module: Cfg, f: &mut FormulaBuilder) -> Vec<Formula> {
    // Stores results.
    let mut verif_conds: Vec<Formula> = Vec::new();

    // Stores cached pre-conditions for each block.
    // Also used to track which blocks have already been visited.
    let mut pre_conds: HashMap<Label, BlockStatus> = HashMap::new();

    // Perform a reverse breadth-first traversal of CFG.
    let mut stack = vec![module.start.clone()];
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
                // If pending, use the requirement if it exists and fail if it doesn't.
                Some(BlockStatus::Pending) | Some(BlockStatus::Cyclic) => {
                    // Mark block as cyclic.
                    *b.unwrap() = BlockStatus::Cyclic;
                    if let Some(c) = &module.blocks[target].require {
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
            Continuation::Exit => Some(module.ensures.clone()),
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
                        f.asym_and(cc.clone(), cond_t),
                        f.asym_and(f.not(cc), cond_f),
                    )
                })
            }
        };
        let post_cond = match post_cond {
            Some(c) => c,
            None => continue,
        };

        // Perform WP-calculus on post-condition with block body.
        let wp_result = wp(f, &block.body, post_cond);

        // Cache or use result of WP.
        let top = f.top();
        let require = block.require.as_ref();
        let require = require.or(if pre_conds[&label] == BlockStatus::Cyclic {
            Some(&top)
        } else {
            None
        });

        if let Some(require) = require {
            // If the block has a requirement,
            // add a VC requiring that the requirement implies the WP result.
            verif_conds.push(f.implies(require.clone(), wp_result));
            pre_conds.insert(label, BlockStatus::PreCond(require.clone()));
        } else {
            // Otherwise, cache the WP result for the block.
            pre_conds.insert(label, BlockStatus::PreCond(wp_result));
        }
    }

    // Add the pre-condition of the starting block as a VC.
    verif_conds.push(match &pre_conds[&module.start] {
        BlockStatus::PreCond(c) => f.implies(module.requires, c.clone()),
        _ => panic!("starting block is never processed"),
    });
    verif_conds
}

fn wp(f: &mut FormulaBuilder, instrs: &[Stmt], mut cond: Formula) -> Formula {
    for instr in instrs.iter().rev() {
        match instr {
            Stmt::Unary(WordSize::B64, op, reg) => {
                let (t, t_id) = f.reg(*reg);
                let e = f.unop(*op, t);
                cond = assign(f, &t_id, e, cond);
            }
            Stmt::Binary(WordSize::B64, op, dst, src) => {
                let (d, d_id) = f.reg(*dst);
                let s = match src {
                    RegImm::Reg(r) => f.reg(*r).0,
                    RegImm::Imm(i) => f.val(*i),
                };
                let e = f.binop(*op, d, s.clone());
                cond = assign(f, &d_id, e, cond);

                // Add extra conditions for division/modulo by zero.
                if op == &BinAlu::Div || op == &BinAlu::Mod {
                    cond = f.asym_and(f.rel(Cc::Ne, s, f.val(0)), cond);
                }
            }
            Stmt::Store(size, mem_ref, _) => {
                let valid_addr = valid_addr(f, *size, mem_ref);
                cond = f.and(valid_addr, cond);
            }
            Stmt::Load(size, dst, mem_ref) => {
                let valid_addr = valid_addr(f, *size, mem_ref);
                let (_, v_id) = f.var(String::from("v"));
                let (_, t_id) = f.reg(*dst);
                let replace_reg = f.forall(v_id.clone(), f.replace(&t_id, &v_id, cond));
                cond = f.and(valid_addr, replace_reg);
            }
            Stmt::Assert(a) => {
                cond = f.asym_and(a.clone(), cond);
            }
            instr => panic!("not implemented: {instr:?}"),
        }
    }
    cond
}

fn assign(f: &mut FormulaBuilder, target: &Ident, e: Expr, cond: Formula) -> Formula {
    let (v, v_id) = f.var(String::from("v"));
    f.forall(
        v_id.clone(),
        f.implies(f.eq(v, e), f.replace(target, &v_id, cond)),
    )
}

fn valid_addr(f: &mut FormulaBuilder, size: WordSize, MemRef(reg, offset): &MemRef) -> Formula {
    let (ptr, ptr_id) = f.var("p".to_owned());
    let (sz, sz_id) = f.var("s".to_owned());
    let addr = f.binop(BinAlu::Add, f.reg(*reg).0, f.val(*offset));
    let bytes = match size {
        WordSize::B8 => 1,
        WordSize::B16 => 2,
        WordSize::B32 => 4,
        WordSize::B64 => 8,
    };
    let upper_bound = f.binop(
        BinAlu::Sub,
        f.binop(BinAlu::Add, ptr.clone(), sz.clone()),
        f.val(bytes - 1),
    );
    f.exists(
        ptr_id.clone(),
        f.exists(
            sz_id,
            f.and(
                f.is_buffer(ptr_id, sz),
                //f.and(
                //    f.eq(f.binop(BinAlu::Mod, addr.clone(), f.val(bytes)), f.val(0)),
                    f.and(
                        f.rel(Cc::Le, ptr, addr.clone()),
                        f.rel(Cc::Lt, addr, upper_bound),
                    ),
                //),
            ),
        ),
    )
}
