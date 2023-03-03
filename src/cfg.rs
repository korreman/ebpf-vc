//! A processed AST, ready for VC-generation.
//! It currently only supports 64-bit operations.

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    mem::swap,
};

pub use crate::ast::{
    BinAlu, Cc, Cont, Expr, Formula, Ident, Imm, Label, MemRef, Offset, Reg, RegImm, Stmt, UnAlu,
    WordSize,
};

use crate::{
    ast::{Line, Logic, Module},
    formula::FormulaBuilder,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Continuation {
    Exit,
    Jmp(Label),
    Jcc(Cc, Reg, RegImm, Label, Label),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub require: Option<Formula>,
    pub body: Vec<Stmt>,
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
    NoExit,
    JumpBounds { target: usize, bound: usize },
    NoLabel(String),
    Unsupported(Stmt),
    MisplacedRequire,
    DuplicateLabel(String),
}

impl Display for ConvertErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConvertErr::NoExit => f.write_fmt(format_args!("Last instruction must be \"exit\"")),
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
                f.write_str("Requirements can only be placed at the start of blocks")
            }
            ConvertErr::DuplicateLabel(label) => {
                f.write_fmt(format_args!("Duplicate label \"{label}\""))
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
    body: Vec<Stmt>,
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

    fn finish(&mut self, next: Continuation) -> Result<(), ConvertErr> {
        if self.blocks.contains_key(&self.label) {
            return Err(ConvertErr::DuplicateLabel(self.label.clone()));
        }
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
        Ok(())
    }

    fn change_label(&mut self, l: Label) {
        let mut tmp = l.clone();
        swap(&mut self.label, &mut tmp);
        self.label_aliases.insert(tmp, l);
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
        if ast.lines.last() != Some(&Line::Cont(Cont::Exit)) {
            return Err(ConvertErr::NoExit);
        }
        for line in ast.lines {
            match line {
                Line::Label(l) => {
                    if !state.body.is_empty() {
                        state.finish(Continuation::Jmp(l.clone()))?;
                    }
                    state.change_label(l);
                }
                Line::Logic(Logic::Assert(a)) => state.body.push(Stmt::Assert(a)),
                Line::Logic(Logic::Require(i)) => {
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
                Line::Stmt(i) => state.body.push(i),
                Line::Cont(c) => match c {
                    // End of blocks
                    Cont::Jmp(t) => {
                        state.finish(Continuation::Jmp(t))?;
                        let next_label = state.next_label();
                        state.change_label(next_label)
                    }
                    Cont::Jcc(cc, reg, reg_imm, target) => {
                        let next_label = state.next_label();
                        state.finish(Continuation::Jcc(
                            cc,
                            reg,
                            reg_imm,
                            target,
                            next_label.clone(),
                        ))?;
                        state.change_label(next_label);
                    }
                    Cont::Exit => state.finish(Continuation::Exit)?,
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
