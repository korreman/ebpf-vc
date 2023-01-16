use crate::ast::{Instr, JmpTarget, Label, Line};
use itertools::Itertools;
use std::collections::HashMap;

pub enum ConvertErr {
    JumpBounds { target: usize, bound: usize },
    NoLabel(String),
    Unsupported(Instr),
}

impl std::fmt::Display for ConvertErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertErr::NoLabel(label) => {
                f.write_fmt(format_args!("Jump target \"{label}\" doesn't exist"))
            }
            ConvertErr::JumpBounds { target, bound } => {
                f.write_fmt(format_args!("Jump target {target} outside bound {bound}"))
            }
            ConvertErr::Unsupported(instr) => {
                f.write_fmt(format_args!("Unsupported instruction: {instr:?}"))
            }
        }
    }
}

impl TryInto<super::Module> for crate::ast::Module {
    type Error = ConvertErr;

    fn try_into(mut self) -> Result<crate::vc::ast::Module, Self::Error> {
        let mut jump_targets: Vec<usize> = vec![0];
        let mut labels: HashMap<Label, usize> = HashMap::new();

        // Collect labels and targets of jumps to use as split indices.
        // Filter out the labels at the same time.
        let mut counter = 0;
        self.retain(|line| match line {
            Line::Label(l) => {
                jump_targets.push(counter);
                labels.insert(l.clone(), counter);
                false
            }
            Line::Instr(i) => {
                match i {
                    Instr::Jmp(JmpTarget::Offset(o)) => {
                        jump_targets.push((counter as i64 + o) as usize + 1)
                    }
                    Instr::Jcc(_, _, _, target) => {
                        jump_targets.push(counter + 1);
                        if let JmpTarget::Offset(o) = target {
                            jump_targets.push((counter as i64 + o) as usize + 1)
                        }
                    }
                    Instr::Exit => jump_targets.push(counter + 1),
                    _ => (),
                }
                counter += 1;
                true
            }
            Line::Assert(_) => todo!(),
        });
        jump_targets.push(self.len() - 1);
        jump_targets.sort();
        jump_targets.dedup();

        // Verify that all jump targets (including offsets) are within program limits.
        let highest_target = *jump_targets.last().unwrap();
        if highest_target > self.len() {
            return Err(ConvertErr::JumpBounds {
                target: highest_target,
                bound: self.len(),
            });
        }

        // Generate tables mapping from line indices to block indices.
        let mut idx_map = HashMap::new();
        for (new_idx, &old_idx) in jump_targets.iter().enumerate() {
            idx_map.insert(old_idx, new_idx);
        }

        // Make labels point to the new block indices instead of line indices.
        for idx in labels.values_mut() {
            *idx = idx_map[idx];
        }

        // Helper closure that converts jump targets to block indices.
        let get_target = |target: &JmpTarget, next| {
            let res = match target {
                JmpTarget::Label(l) => labels.get(l).ok_or(ConvertErr::NoLabel(l.clone())),
                JmpTarget::Offset(o) => Ok(&idx_map[&((next as i64 + *o) as usize)]),
            };
            res.cloned()
        };

        // Slice the vector into blocks according to these jump targets.
        // Take each slice and package up as a block.
        // - Convert instructions to limited subset.
        // - Resolve jump targets to block indices
        // - If a block doesn't end with a jump or exit, create a jump to the succeeding block.
        let blocks: Result<Vec<super::Block>, ConvertErr> = jump_targets
            .iter()
            .tuple_windows()
            .map(|(&idx, &next)| -> Result<super::Block, ConvertErr> {
                let mut slice = &self[idx..next];

                // Generate the appropriate continuation for the block.
                let mut last_is_not_cont = false;
                let next = match slice.last().expect("no slices") {
                    Line::Instr(i) => match i {
                        Instr::Jmp(t) => super::Continuation::Jmp(get_target(t, next)?),
                        Instr::Jcc(cc, reg, reg_imm, target) => {
                            let target_t = get_target(target, next)?;
                            let target_f = get_target(&JmpTarget::Offset(0), next)?;
                            super::Continuation::Jcc(*cc, *reg, *reg_imm, target_t, target_f)
                        }
                        Instr::Exit => super::Continuation::Exit,
                        _ => {
                            last_is_not_cont = true;
                            super::Continuation::Jmp(get_target(&JmpTarget::Offset(0), next)?)
                        }
                    },
                    Line::Label(_) => panic!("labels should've been filtered by now"),
                    Line::Assert(_) => todo!(),
                };

                // Remove the last instruction from the body if it's a continuation.
                if !last_is_not_cont {
                    slice = &slice[..slice.len() - 1];
                }

                // Convert the body to constrained instruction set.
                let body: Vec<super::Instr> = slice
                    .iter()
                    .map(|l| match l {
                        Line::Instr(i) => match i {
                            Instr::Unary(s, o, r) => super::Instr::Unary(*s, *o, *r),
                            Instr::Binary(s, o, d, ri) => super::Instr::Binary(*s, *o, *d, *ri),
                            Instr::Store(s, m, ri) => super::Instr::Store(*s, *m, *ri),
                            Instr::Load(s, d, m) => super::Instr::Load(*s, *d, *m),
                            Instr::LoadImm(r, i) => super::Instr::LoadImm(*r, *i),
                            Instr::LoadMapFd(r, i) => super::Instr::LoadMapFd(*r, *i),
                            Instr::Call(i) => super::Instr::Call(*i),
                            instr => panic!("no case for {instr:?}"),
                        },
                        Line::Label(_) => panic!("labels should've been filtered by now"),
                        Line::Assert(_) => todo!(),
                    })
                    .collect();

                Ok(super::Block {
                    pre_assert: None,
                    body,
                    next,
                })
            })
            .collect();
        Ok(super::Module { blocks: blocks? })
    }
}
