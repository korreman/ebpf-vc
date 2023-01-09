//! Parsing of eBPF assembly.
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, newline, one_of, satisfy, space0},
    combinator::{complete, map, map_opt, map_res, opt, recognize, value},
    multi::{many0, many0_count, many1, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};
type Res<'a, O> = IResult<&'a str, O>;

#[cfg(test)]
mod tests;

use crate::ast::*;

// Tokens

fn num(i: &str) -> Res<i64> {
    let num_dec = map_res(
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        |out: &str| str::replace(out, "_", "").parse::<i64>(),
    );

    let num_bin = map_res(
        preceded(
            tag("0b"),
            recognize(many1(terminated(one_of("01"), many0(char('_'))))),
        ),
        |out: &str| i64::from_str_radix(&str::replace(out, "_", ""), 2),
    );

    let num_hex = map_res(
        preceded(
            tag("0x"),
            recognize(many1(terminated(
                one_of("0123456789abcdefABCDEF"),
                many0(char('_')),
            ))),
        ),
        |out: &str| i64::from_str_radix(&str::replace(out, "_", ""), 16),
    );

    alt((num_hex, num_bin, num_dec))(i)
}

fn ident(i: &str) -> Res<&str> {
    recognize(tuple((
        opt(tag("_")),
        alpha1,
        many0_count(alt((alphanumeric1, tag("_")))),
    )))(i)
}

// Instruction parsing

/// Separator between components of an instruction
fn isep(i: &str) -> Res<()> {
    value((), tuple((many0(char(' ')), char(','), many0(char(' ')))))(i)
}

// TODO: 64 bit should be an empty string
fn alu_size(i: &str) -> Res<WordSize> {
    alt((
        value(WordSize::B32, tag("32")),
        value(WordSize::B64, tag("64")),
    ))(i)
}

fn reg(i: &str) -> Res<Reg> {
    map_opt(preceded(char('r'), one_of("0123456789")), |c| {
        Some(Reg::new(c.to_digit(10)? as usize))?
    })(i)
}

fn imm(i: &str) -> Res<Imm> {
    // TODO: Negative numbers
    num(i)
}

fn reg_imm(i: &str) -> Res<RegImm> {
    alt((map(reg, RegImm::Reg), map(imm, RegImm::Imm)))(i)
}

fn offset(i: &str) -> Res<Offset> {
    alt((
        preceded(pair(char('+'), space0), num),
        preceded(pair(char('-'), space0), map(num, |n| -n)),
    ))(i)
}

fn unary(i: &str) -> Res<Instr> {
    let op = alt((
        value(UnAlu::Neg, tag("neg")),
        value(UnAlu::Le, tag("le")),
        value(UnAlu::Be, tag("be")),
    ));
    map(tuple((op, alu_size, isep, reg)), |(op, size, _, reg)| {
        Instr::Unary(size, op, reg)
    })(i)
}

fn binary(i: &str) -> Res<Instr> {
    let op = alt((
        value(BinAlu::Add, tag("add")),
        value(BinAlu::Sub, tag("sub")),
        value(BinAlu::Mul, tag("mul")),
        value(BinAlu::Div, tag("div")),
        value(BinAlu::Mod, tag("mod")),
        value(BinAlu::And, tag("and")),
        value(BinAlu::Or, tag("or")),
        value(BinAlu::Xor, tag("xor")),
        value(BinAlu::Lsh, tag("lsh")),
        value(BinAlu::Rsh, tag("rsh")),
        value(BinAlu::Arsh, tag("arsh")),
    ));
    map(
        tuple((op, alu_size, isep, reg, isep, reg_imm)),
        |(op, size, _, reg, _, reg_imm)| Instr::Binary(size, op, reg, reg_imm),
    )(i)
}

fn mem_size(i: &str) -> Res<WordSize> {
    alt((
        value(WordSize::B8, char('b')),
        value(WordSize::B16, char('h')),
        value(WordSize::B32, char('w')),
        value(WordSize::B64, tag("dw")),
    ))(i)
}

fn mem_ref(i: &str) -> Res<MemRef> {
    let inner = map(tuple((reg, space0, opt(offset))), |(reg, _, offset)| {
        MemRef(reg, offset)
    });
    delimited(char('['), inner, char(']'))(i)
}

fn load(i: &str) -> Res<Instr> {
    map(
        tuple((tag("ldx"), mem_size, isep, reg, isep, mem_ref)),
        |(_, size, _, reg, _, mref)| Instr::Load(size, reg, mref),
    )(i)
}

fn store(i: &str) -> Res<Instr> {
    let inner_imm = map(
        tuple((mem_size, isep, mem_ref, isep, imm)),
        |(size, _, mref, _, imm)| (size, mref, RegImm::Imm(imm)),
    );

    let inner_reg = map(
        preceded(char('x'), tuple((mem_size, isep, mem_ref, isep, reg))),
        |(size, _, mref, _, reg)| (size, mref, RegImm::Reg(reg)),
    );

    map(
        preceded(tag("st"), alt((inner_reg, inner_imm))),
        |(size, mref, reg_imm)| Instr::Store(size, mref, reg_imm),
    )(i)
}

fn jmp_target(i: &str) -> Res<JmpTarget> {
    alt((
        map(ident, |l| JmpTarget::Label(l.to_owned())),
        map(offset, JmpTarget::Offset),
    ))(i)
}

fn jcc(i: &str) -> Res<Instr> {
    let cc = alt((
        value(Cc::Eq, tag("eq")),
        value(Cc::Gt, tag("gt")),
        value(Cc::Ge, tag("ge")),
        value(Cc::Lt, tag("lt")),
        value(Cc::Le, tag("le")),
        value(Cc::Set, tag("set")),
        value(Cc::Ne, tag("ne")),
        value(Cc::Sgt, tag("sgt")),
        value(Cc::Sge, tag("sge")),
        value(Cc::Slt, tag("slt")),
        value(Cc::Sle, tag("sle")),
    ));
    map(
        tuple((char('j'), cc, isep, reg, isep, reg_imm, jmp_target)),
        |(_, cc, _, lhs, _, rhs, target)| Instr::Jcc(cc, lhs, rhs, target),
    )(i)
}

fn instr(i: &str) -> Res<Instr> {
    let jmp = map(preceded(pair(tag("jmp"), isep), jmp_target), Instr::Jmp);
    let call = map(preceded(pair(tag("call"), isep), imm), Instr::Call);
    let load_imm = map(preceded(pair(tag("lddw"), isep), imm), Instr::LoadImm);
    let exit = value(Instr::Exit, tag("exit"));
    // Missing: LoadMapFd
    alt((unary, binary, load, load_imm, store, jcc, jmp, call, exit))(i)
}

// Structural parsing

fn line_sep(i: &str) -> Res<()> {
    value(
        (),
        many1(tuple((
            space0,
            opt(pair(char(';'), many0(satisfy(|c| c != '\n')))),
            newline,
            space0,
        ))),
    )(i)
}

fn label(i: &str) -> Res<Label> {
    terminated(ident, pair(space0, tag(":")))
        .map(|l| l.to_owned())
        .parse(i)
}

fn line(i: &str) -> Res<Line> {
    label.map(Line::Label).or(instr.map(Line::Instr)).parse(i)
}

pub fn module(i: &str) -> Res<Module> {
    complete(separated_list0(line_sep, line))(i)
}
