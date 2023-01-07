//! A processed AST, ready for VC-generation.

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
pub enum WordSize {
    B8,
    B16,
    B32,
    B64,
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
    pub precond: Option<Formula>,
    pub body: Vec<Instr>,
    pub next: Continuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub blocks: Vec<Block>,
}
