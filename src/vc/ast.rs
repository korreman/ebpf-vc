//! A processed AST, ready for VC-generation.
//! It currently only supports 64-bit operations.

use std::collections::HashMap;

pub use crate::ast::{BinAlu, Cc, Imm, MemRef, Offset, Reg, RegImm, UnAlu};
use crate::ast::{Formula, Label, WordSize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr {
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
    pub body: Vec<Instr>,
    pub next: Continuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub requires: Formula,
    pub ensures: Formula,
    pub start: Label,
    pub blocks: HashMap<Label, Block>,
}
