//! A processed AST, ready for VC-generation.
//! It currently only supports 64-bit operations.

use crate::logic::Formula;
pub use crate::ast::{Cc, UnAlu, BinAlu, Reg, Imm, RegImm, Offset, MemRef};

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
