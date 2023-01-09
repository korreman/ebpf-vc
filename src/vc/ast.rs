//! A processed AST, ready for VC-generation.
//! It currently only supports 64-bit operations.

pub use crate::ast::{BinAlu, Cc, Imm, MemRef, Offset, Reg, RegImm, UnAlu};
use crate::logic::Formula;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instr {
    Unary(UnAlu, Reg),
    Binary(BinAlu, Reg, RegImm),
    Store(MemRef, RegImm),
    Load(Reg, MemRef),
}

pub type Label = usize;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Continuation {
    Exit,
    Jmp(Label),
    Jcc(Cc, Reg, RegImm, Label, Label),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub pre_assert: Option<Formula>,
    pub body: Vec<Instr>,
    pub next: Continuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    /// Collection of program blocks.
    /// These should be arranged as they occur in the code,
    /// so the 0'th block is the entry-point of the module.
    pub blocks: Vec<Block>,
}

pub use preprocess::*;
mod preprocess {
    use crate::ast::{Instr, JmpTarget, Label, Line, WordSize};
    use itertools::Itertools;
    use std::collections::HashMap;

    pub enum ConvertErr {
        Invalid,
        Unsupported,
        Internal,
    }

    impl TryInto<super::Module> for crate::ast::Module {
        type Error = ConvertErr;

        fn try_into(mut self) -> Result<crate::vc::ast::Module, Self::Error> {
            // Indices denoting the start of blocks.
            let mut block_idxs: Vec<usize> = Vec::new();

            // Collect label indices.
            let mut label_idxs: HashMap<Label, usize> = HashMap::new();
            let mut idx_labels: HashMap<usize, Label> = HashMap::new();
            let mut counter = 0;
            self.retain(|line| {
                if let Line::Label(label) = line {
                    block_idxs.push(counter);
                    label_idxs.insert(label.clone(), counter);
                    idx_labels.insert(counter, label.clone());
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
                        Instr::Jmp(target) => {
                            if let JmpTarget::Offset(o) = target {
                                block_idxs.push((idx as i64 + o) as usize + 1)
                            }
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
            block_idxs.sort();
            block_idxs.push(self.len() - 1);
            block_idxs.dedup();
            // Check all indices (including offsets) are within program limits.
            if *block_idxs.last().unwrap_or(&0) >= self.len() {
                return Err(ConvertErr::Invalid);
            }

            // Generate tables mapping from line indices/labels to block indices.
            let mut idx_map = HashMap::new();
            for (new_idx, &old_idx) in block_idxs.iter().enumerate() {
                idx_map.insert(old_idx, new_idx);
            }
            for (_, idx) in &mut label_idxs {
                *idx = idx_map[idx];
            }
            let get_target = |target: &JmpTarget, next| {
                let res = match target {
                    JmpTarget::Label(l) => label_idxs.get(l),
                    JmpTarget::Offset(o) => idx_map.get(&((next as i64 + *o) as usize)),
                };
                res.cloned().ok_or(ConvertErr::Invalid)
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
                    let next = match slice.last().ok_or(ConvertErr::Invalid)? {
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
                        Line::Label(_) => return Err(ConvertErr::Internal),
                    };

                    if !last_is_not_cont {
                        slice = &slice[..slice.len() - 1];
                    }

                    let body: Result<Vec<super::Instr>, ConvertErr> = slice
                        .iter()
                        .map(|l| match l {
                            Line::Label(_) => Err(ConvertErr::Internal),
                            Line::Instr(i) => match i {
                                Instr::Unary(WordSize::B64, op, reg) => {
                                    Ok(super::Instr::Unary(*op, *reg))
                                }
                                Instr::Binary(WordSize::B64, op, dst, src) => {
                                    Ok(super::Instr::Binary(*op, *dst, *src))
                                }
                                Instr::Store(WordSize::B64, mref, reg_imm) => {
                                    Ok(super::Instr::Store(*mref, *reg_imm))
                                }
                                Instr::Load(WordSize::B64, dst, mref) => {
                                    Ok(super::Instr::Load(*dst, *mref))
                                }
                                _ => Err(ConvertErr::Internal),
                            },
                        })
                        .collect();

                    Ok(super::Block {
                        pre_assert: None,
                        body: body?,
                        next,
                    })
                })
                .collect();
            Ok(super::Module { blocks: blocks? })
        }
    }
}
