use crate::ast::{FBinOp, Formula, FormulaLine, Instr, Label, Line};
use std::{collections::HashMap, mem::swap};

use super::ast::{Block, Continuation};

pub enum ConvertErr {
    JumpBounds { target: usize, bound: usize },
    NoLabel(String),
    Unsupported(Instr),
    MisplacedRequire,
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
            ConvertErr::MisplacedRequire => {
                f.write_str("requirements can only be placed at the start of blocks")
            }
        }
    }
}

struct State {
    blocks: HashMap<Label, Block>,
    label_aliases: HashMap<String, String>,
    label_counter: usize,
    label: String,
    require: Option<Formula>,
    body: Vec<super::Instr>,
}

impl State {
    fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            label_aliases: HashMap::new(),
            label: "@0".to_owned(),
            label_counter: 0,
            require: None,
            body: Vec::new(),
        }
    }

    fn finish(&mut self, next: Continuation) {
        let mut label = "".to_owned();
        let mut require = None;
        let mut body = Vec::new();
        swap(&mut self.label, &mut label);
        swap(&mut self.require, &mut require);
        swap(&mut self.body, &mut body);
        self.blocks.insert(
            label,
            Block {
                require,
                body,
                next,
            },
        );
    }

    fn change_label(&mut self, mut l: Label) {
        // TODO: resolve multiple occurrences of same label
        let value = l.clone();
        swap(&mut self.label, &mut l);
        self.label_aliases.insert(l, value);
    }

    fn next_label(&mut self) -> Label {
        self.label_counter += 1;
        format!("@{}", self.label_counter)
    }

    fn resolve_aliases(&mut self) {
        let resolve = |target: &mut Label| {
            if let Some(l) = self.label_aliases.get(target) {
                *target = l.clone();
            }
        };
        for block in self.blocks.values_mut() {
            match &mut block.next {
                Continuation::Jcc(_, _, _, target_t, target_f) => {
                    resolve(target_t);
                    resolve(target_f);
                }
                Continuation::Jmp(target) => {
                    resolve(target);
                }
                _ => (),
            }
        }
    }
}

impl TryInto<super::Module> for crate::ast::Module {
    type Error = ConvertErr;

    fn try_into(self) -> Result<crate::vc::ast::Module, Self::Error> {
        // TODO: Jump followed by targets point to wrong internal name.
        let mut state = State::new();
        for line in self {
            match line {
                Line::Label(l) => {
                    if !state.body.is_empty() {
                        state.finish(Continuation::Jmp(l.clone()));
                    }
                    state.change_label(l);
                }
                Line::Formula(FormulaLine::Assert(a)) => state.body.push(super::Instr::Assert(a)),
                Line::Formula(FormulaLine::Require(i)) => {
                    if state.body.is_empty() {
                        state.require = match state.require {
                            // TODO: Normal or asymmetric conjugation?
                            Some(pa) => Some(Formula::Bin(FBinOp::AndAsym, Box::new((pa, i)))),
                            None => Some(i),
                        };
                    } else {
                        return Err(ConvertErr::MisplacedRequire);
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
                        let next_label = state.next_label();
                        state.change_label(next_label)
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
                        state.change_label(next_label);
                    }
                    Instr::Exit => state.finish(Continuation::Exit),
                },
            }
        }
        state.resolve_aliases();
        Ok(super::ast::Module {
            start: state
                .label_aliases
                .get("@0")
                .unwrap_or(&"@0".to_owned())
                .to_owned(),
            blocks: state.blocks,
        })
    }
}
