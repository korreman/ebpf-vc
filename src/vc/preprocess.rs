use crate::ast::{Instr, JmpTarget, Label, Line};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

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
        // Indices denoting the start of blocks.
        let mut block_idxs: Vec<usize> = vec![0];

        // Collect label indices.
        let mut label_idxs: HashMap<Label, usize> = HashMap::new();
        let mut counter = 0;
        self.retain(|line| {
            if let Line::Label(label) = line {
                block_idxs.push(counter);
                label_idxs.insert(label.clone(), counter);
                false
            } else {
                counter += 1;
                true
            }
        });

        // Collect jump target indices.
        for (idx, line) in self.iter().enumerate() {
            if let Line::Instr(instr) = line {
                match instr {
                    Instr::Jmp(JmpTarget::Offset(o)) => {
                        block_idxs.push((idx as i64 + o) as usize + 1)
                    }
                    Instr::Jcc(_, _, _, target) => {
                        block_idxs.push(idx + 1);
                        if let JmpTarget::Offset(o) = target {
                            block_idxs.push((idx as i64 + o) as usize + 1)
                        }
                    }
                    Instr::Exit => block_idxs.push(idx + 1),
                    _ => (),
                }
            }
        }
        block_idxs.push(self.len() - 1);
        block_idxs.sort();
        block_idxs.dedup();

        // TODO: Don't check if there are no blocks
        // Check all indices (including offsets) are within program limits.
        if let Some(&highest_idx) = block_idxs.last() {
            if highest_idx > self.len() {
                return Err(ConvertErr::JumpBounds {
                    target: highest_idx,
                    bound: self.len(),
                });
            }
        }

        // Generate tables mapping from line indices/labels to block indices.
        let mut idx_map = HashMap::new();
        for (new_idx, &old_idx) in block_idxs.iter().enumerate() {
            idx_map.insert(old_idx, new_idx);
        }
        for idx in label_idxs.values_mut() {
            *idx = idx_map[idx];
        }
        let get_target = |target: &JmpTarget, next| {
            let res = match target {
                JmpTarget::Label(l) => label_idxs.get(l).ok_or(ConvertErr::NoLabel(l.clone())),
                JmpTarget::Offset(o) => Ok(&idx_map[&((next as i64 + *o) as usize)]),
            };
            res.cloned()
        };

        // Slice the vector into blocks according to these indices.
        // Take each slice and package up as a block.
        // - Resolve jump targets to block indices
        // - If a block doesn't end with a jump or exit, create a jump to the succeeding block.
        let blocks: Result<Vec<super::Block>, ConvertErr> = block_idxs
            .iter()
            .tuple_windows()
            .map(|(&idx, &next)| -> Result<super::Block, ConvertErr> {
                let mut slice = &self[idx..next];
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
                };

                if !last_is_not_cont {
                    slice = &slice[..slice.len() - 1];
                }

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
