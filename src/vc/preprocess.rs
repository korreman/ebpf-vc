use crate::ast::{FBinOp, Formula, Instr, Label, Line};
use std::{collections::HashMap, mem::swap};

use super::ast::{Block, Continuation};

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

struct State {
    blocks: HashMap<Label, Block>,
    label_counter: usize,
    label: String,
    pre_assert: Option<Formula>,
    body: Vec<super::Instr>,
}

impl State {
    fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            label: "@0".to_owned(),
            label_counter: 0,
            pre_assert: None,
            body: Vec::new(),
        }
    }

    fn finish(&mut self, next: Continuation) {
        let mut label = "".to_owned();
        let mut pre_assert = None;
        let mut body = Vec::new();
        swap(&mut self.label, &mut label);
        swap(&mut self.pre_assert, &mut pre_assert);
        swap(&mut self.body, &mut body);
        self.blocks.insert(
            label,
            Block {
                pre_assert,
                body,
                next,
            },
        );
    }

    fn next_label(&mut self) -> Label {
        self.label_counter += 1;
        format!("@{}", self.label_counter)
    }
}

impl TryInto<super::Module> for crate::ast::Module {
    type Error = ConvertErr;

    fn try_into(self) -> Result<crate::vc::ast::Module, Self::Error> {
        let mut state = State::new();
        for line in self {
            match line {
                Line::Label(l) => {
                    if !state.body.is_empty() {
                        state.finish(Continuation::Jmp(l.clone()));
                    }
                    state.label = l;
                }
                Line::Assert(a) => {
                    if state.body.is_empty() {
                        state.pre_assert = match state.pre_assert {
                            // TODO: Normal or asymmetric conjugation?
                            Some(pa) => Some(Formula::Bin(FBinOp::AndAsym, Box::new((pa, a)))),
                            None => Some(a),
                        };
                    } else {
                        state.body.push(super::Instr::Assert(a))
                    }
                }
                Line::Instr(i) => match i {
                    // Simple conversions
                    Instr::Unary(s, o, r) => state.body.push(super::Instr::Unary(s, o, r)),
                    Instr::Binary(s, o, d, ri) => {
                        state.body.push(super::Instr::Binary(s, o, d, ri))
                    }
                    Instr::Store(s, m, ri) => state.body.push(super::Instr::Store(s, m, ri)),
                    Instr::Load(s, d, m) => state.body.push(super::Instr::Load(s, d, m)),
                    Instr::LoadImm(r, i) => state.body.push(super::Instr::LoadImm(r, i)),
                    Instr::LoadMapFd(r, i) => state.body.push(super::Instr::LoadMapFd(r, i)),
                    Instr::Call(i) => state.body.push(super::Instr::Call(i)),
                    // End of blocks
                    Instr::Jmp(t) => {
                        state.finish(Continuation::Jmp(t));
                        state.label = state.next_label()
                    }
                    Instr::Jcc(cc, reg, reg_imm, target) => {
                        let next_label = state.next_label();
                        state.finish(Continuation::Jcc(
                            cc,
                            reg,
                            reg_imm,
                            target,
                            next_label.clone(),
                        ));
                        state.label = next_label;
                    }
                    Instr::Exit => state.finish(Continuation::Exit),
                },
            }
        }
        Ok(super::ast::Module {
            start: "@0".to_owned(),
            blocks: state.blocks,
        })
    }
}
