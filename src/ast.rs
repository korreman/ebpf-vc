//! An AST used for parsing eBPF assembly.

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordSize {
    B8, B16, B32, B64,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cc {
    Eq, Gt, Ge, Lt, Le, Set, Ne, Sgt, Sge, Slt, Sle,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnAlu {
    Neg, Le, Be,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinAlu {
    Mov, Add, Sub, Mul, Div, Mod, And, Or, Xor, Lsh, Rsh, Arsh,
}

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

pub type Imm = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegImm {
    Reg(Reg),
    Imm(Imm),
}

pub type Offset = i64;
pub type Label = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemRef(pub Reg, pub Option<Offset>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JmpTarget {
    Label(Label),
    Offset(Offset),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr {
    Unary(WordSize, UnAlu, Reg),
    Binary(WordSize, BinAlu, Reg, RegImm),
    Store(WordSize, MemRef, RegImm),
    Load(WordSize, Reg, MemRef),
    LoadImm(Imm),
    LoadMapFd(Reg, Imm),
    Jmp(JmpTarget),
    Jcc(Cc, Reg, RegImm, JmpTarget, JmpTarget),
    Call(Imm),
    Exit,
}

pub enum Line {
    Label(Label),
    Instr(Instr),
    // TODO: Assertions
}

pub type Module = Vec<Line>;
