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
pub struct Reg(u8);
impl Reg {
    pub const R0: Self = Reg(0);
    pub const R1: Self = Reg(1);
    pub const R2: Self = Reg(2);
    pub const R3: Self = Reg(3);
    pub const R4: Self = Reg(4);
    pub const R5: Self = Reg(5);
    pub const R6: Self = Reg(6);
    pub const R7: Self = Reg(7);
    pub const R8: Self = Reg(8);
    pub const R9: Self = Reg(9);

    pub fn new(id: u8) -> Option<Self> {
        if id < 10 {
            Some(Self(id))
        } else {
            None
        }
    }

    pub fn get(&self) -> u8 {
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
pub struct MemRef(pub Reg, pub Offset);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr {
    Unary(WordSize, UnAlu, Reg),
    Binary(WordSize, BinAlu, Reg, RegImm),
    Store(WordSize, MemRef, RegImm),
    Load(WordSize, Reg, MemRef),
    LoadImm(Reg, Imm),
    LoadMapFd(Reg, Imm),
    Jmp(Label),
    Jcc(Cc, Reg, RegImm, Label),
    Call(Imm),
    Exit,
}

pub type Ident = String;

/// Expression used for formulas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Val(Imm),
    Var(Ident),
    Unary(UnAlu, Box<Expr>),
    Binary(BinAlu, Box<(Expr, Expr)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FBinOp {
    And,
    Or,
    Implies,
    Iff,
    AndAsym,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QType {
    Exists,
    Forall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Formula {
    Val(bool),
    Not(Box<Formula>),
    Bin(FBinOp, Box<(Formula, Formula)>),
    Quant(QType, Ident, Box<Formula>),
    Rel(Cc, Expr, Expr),
    IsBuffer(Ident, Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaLine {
    Assert(Formula),
    Require(Formula),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Label(Label),
    Formula(FormulaLine),
    Instr(Instr),
}

pub struct Module {
    pub requires: Vec<Formula>,
    pub ensures: Vec<Formula>,
    pub lines: Vec<Line>,
}
