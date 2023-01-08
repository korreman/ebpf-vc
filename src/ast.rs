//! A processed AST, ready for VC-generation.
//! It currently only supports 64-bit operations.

use crate::logic::Formula;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reg(usize);
impl Reg {
    pub fn new(id: usize) -> Option<Self> {
        if id < 10 {
            Some(Self(id))
        } else {
            None
        }
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cc {
    Eq,
    Gt,
    Ge,
    Lt,
    Le,
    Set,
    Ne,
    Sgt,
    Sge,
    Slt,
    Sle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnAlu {
    Neg,
    Le,
    Be,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinAlu {
    Mov,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Lsh,
    Rsh,
    Arsh,
}

pub type Imm = i64;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegImm {
    Reg(Reg),
    Imm(Imm),
}
pub type Offset = i64;
pub type MemRef = (Reg, Option<Offset>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instr {
    Unary(UnAlu, Reg),
    Binary(BinAlu, Reg, RegImm),
    Store(MemRef, RegImm),
    Load(Reg, MemRef),
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
