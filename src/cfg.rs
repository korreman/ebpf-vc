//! A processed AST, ready for VC-generation.
//! It currently only supports 64-bit operations.

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    mem::swap,
};

pub use crate::ast::{
    BinAlu, Cc, Expr, Formula, Ident, Imm, Label, MemRef, Offset, Reg, RegImm, UnAlu, WordSize,
};
use crate::{
    ast::{FormulaLine, Instr, Line, Module},
    formula::FormulaBuilder,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CInstr {
    Assert(Formula),
    Unary(WordSize, UnAlu, Reg),
    Binary(WordSize, BinAlu, Reg, RegImm),
    Store(WordSize, MemRef, RegImm),
    Load(WordSize, Reg, MemRef),
    LoadImm(Reg, Imm),
    LoadMapFd(Reg, Imm),
    Call(Imm),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Continuation {
    Exit,
    Jmp(Label),
    Jcc(Cc, Reg, RegImm, Label, Label),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub require: Option<Formula>,
    pub body: Vec<CInstr>,
    pub next: Continuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cfg {
    pub requires: Formula,
    pub ensures: Formula,
    pub start: Label,
    pub blocks: HashMap<Label, Block>,
}

pub enum ConvertErr {
    JumpBounds { target: usize, bound: usize },
    NoLabel(String),
    Unsupported(Instr),
    MisplacedRequire,
}

impl Display for ConvertErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    body: Vec<CInstr>,
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

impl Cfg {
    pub fn create(ast: Module, f: &mut FormulaBuilder) -> Result<Cfg, ConvertErr> {
        let mut state = State::new();
        for line in ast.lines {
            match line {
                Line::Label(l) => {
                    if !state.body.is_empty() {
                        state.finish(Continuation::Jmp(l.clone()));
                    }
                    state.change_label(l);
                }
                Line::Formula(FormulaLine::Assert(a)) => state.body.push(CInstr::Assert(a)),
                Line::Formula(FormulaLine::Require(i)) => {
                    if state.body.is_empty() {
                        state.require = match state.require {
                            // TODO: Normal or asymmetric conjugation?
                            Some(pa) => Some(f.asym_and(pa, i)),
                            None => Some(i),
                        };
                    } else {
                        return Err(ConvertErr::MisplacedRequire);
                    }
                }
                Line::Instr(i) => match i {
                    // Simple conversions
                    Instr::Unary(s, o, r) => state.body.push(CInstr::Unary(s, o, r)),
                    Instr::Binary(s, o, d, ri) => state.body.push(CInstr::Binary(s, o, d, ri)),
                    Instr::Store(s, m, ri) => state.body.push(CInstr::Store(s, m, ri)),
                    Instr::Load(s, d, m) => state.body.push(CInstr::Load(s, d, m)),
                    Instr::LoadImm(r, i) => state.body.push(CInstr::LoadImm(r, i)),
                    Instr::LoadMapFd(r, i) => state.body.push(CInstr::LoadMapFd(r, i)),
                    Instr::Call(i) => state.body.push(CInstr::Call(i)),
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

        let requires = ast.requires.into_iter().fold(f.top(), |a, b| f.and(a, b));
        let ensures = ast.ensures.into_iter().fold(f.top(), |a, b| f.and(a, b));
        Ok(Cfg {
            requires,
            ensures,
            start: state
                .label_aliases
                .get("@0")
                .unwrap_or(&"@0".to_owned())
                .to_owned(),
            blocks: state.blocks,
        })
    }
}
