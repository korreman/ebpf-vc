//! A processed AST, ready for VC-generation.
//! It currently only supports 64-bit operations.

pub use crate::ast::{BinAlu, Cc, Imm, MemRef, Offset, Reg, RegImm, UnAlu};
use crate::{ast::WordSize, logic::Formula};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instr {
    Unary(WordSize, UnAlu, Reg),
    Binary(WordSize, BinAlu, Reg, RegImm),
    Store(WordSize, MemRef, RegImm),
    Load(WordSize, Reg, MemRef),
    LoadImm(Reg, Imm),
    LoadMapFd(Reg, Imm),
    Call(Imm),
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
