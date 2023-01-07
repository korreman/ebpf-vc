//! Verification condition generation.

use crate::ast::*;
use crate::logic::*;

pub fn vc(module: Module) -> Formula {
    // 1. Collect all blocks that end with an exit.
    // 2. Perform a reverse breadth-first traversal of CFG. For each block:
    // a. Acquire the correct post-conditions for that block.
    //    Exit: No conditions, just True.
    //    Jmp: Pre-condition of the target block.
    //    Jcc: Conjunction of the pre-conditions of both targets,
    //         with the jump condition tacked on to each respectively.
    // b. Perform WP on the precondition to obtain P.
    // c.
    //    - If the block has no asserted preconditions,
    //      assign P to the block as a precondition.
    //    - If the block does have asserted preconditions A,
    //      require (somewhere?) that P => A.
    //    - If the block as already been visited but has no asserted precondition, abort.
    // c. Find all blocks that may jump to the current one,
    //    add them to the next round.
    let mut labels: Vec<Label> = module
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block.next == Continuation::Exit)
        .map(|(label, _)| label)
        .collect();
    let mut next_labels: Vec<usize> = Vec::new();

    while !labels.is_empty() {
        for label in labels.drain(..) {
            let block = &module.blocks[label];
            let post_cond = f_true(); // TODO
        }
    }

    todo!()
}

fn wp(instrs: Vec<Instr>, mut cond: Formula) -> Formula {
    for instr in instrs.iter().rev() {
        match instr {
            // I need to generate a new variable.
            Instr::Unary(size, op, target) => {
                let ts = reg_to_ident(*target);
                let te = reg_to_var(*target);
                let v = Expr::Var(String::from("v")); // TODO: i have to create a brand new variable
                let e = e_unop(*op, te);
                cond = f_forall(ts, f_implies(f_eq(v, e), cond))
            }
            Instr::Binary(_, _, _, _) => todo!(),
            Instr::Store(_, _, _) => todo!(),
            Instr::Load(_, _, _) => todo!(),
            Instr::LoadImm(_, _) => todo!(),
            Instr::LoadMapFd(_, _) => todo!(),
            Instr::Call(_) => todo!(),
        }
    }
    cond
}
