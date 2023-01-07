//! An AST for eBPF code.

// Question:
// Should I make a separate AST for parsed input?
// I guess we can start by working with an AST that's ready to be parsed.

pub struct Reg(usize);

impl Reg {
    pub fn new(id: u32) -> Option<Self> {
        if id < 10 {
            Some(Self { id })
        } else {
            None
        }
    }
}

pub enum WordSize {
    B8,
    B16,
    B32,
    B64,
}

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

pub enum UnAlu {
    Neg,
    Le,
    Be,
}

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
pub type Offset = i64;
pub type MemRef = (Reg, Option<Offset>);
pub enum RegImm {
    Reg(Reg),
    Imm(Imm),
}

pub enum Instr {
    Unary(WordSize, UnAlu, Reg),
    Binary(WordSize, BinAlu, Reg, RegImm),
    Store(WordSize, MemRef, RegImm),
    Load(WordSize, Reg, MemRef),
    LoadImm(Reg, Imm),
    LoadMapFd(Reg, Imm),
    Call(Imm),
}

pub type Label = String;
pub enum Continuation {
    Exit,
    Jmp(Label),
    Jcc(Cc, Reg, RegImm, Label, Label),
}

pub struct Block {
    // TODO: Precondition
    pub body: Vec<Instr>,
    pub next: Continuation,
}

pub struct Module {
    pub blocks: HashMap<Label, Block>,
}
