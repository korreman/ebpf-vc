//! Parsing of eBPF assembly.
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, newline, one_of, satisfy, space0, space1},
    combinator::{eof, map, map_opt, map_res, opt, peek, recognize, value, verify},
    multi::{many0, many0_count, many1, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};
type Res<'a, O> = IResult<&'a str, O>;

#[cfg(test)]
#[rustfmt::skip]
mod tests;

use crate::ast::*;

// TODO: Improve the whitespace story.

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

fn reg(i: &str) -> Res<Reg> {
    map_opt(preceded(char('r'), one_of("0123456789")), |c| {
        Some(Reg::new(c.to_digit(10)? as u8))?
    })(i)
}

fn imm(i: &str) -> Res<Imm> {
    pair(opt(terminated(alt((char('+'), char('-'))), space0)), num)
        .map(|(sign, n)| match sign {
            Some('+') | None => n,
            Some('-') => -n,
            _ => unreachable!(),
        })
        .parse(i)
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

fn alu_size(i: &str) -> Res<WordSize> {
    alt((
        value(WordSize::B32, tag("32")),
        value(WordSize::B64, opt(tag("64"))),
    ))(i)
}

/// Separator between components of an instruction
fn isep(i: &str) -> Res<()> {
    value(
        (),
        verify(
            recognize(tuple((space0, opt(char(',')), space0))),
            |res: &str| !res.is_empty(),
        ),
    )(i)
}

macro_rules! instr {
    ( $head:expr, $first:expr $(, $($tail:expr),* )? ) => {
        tuple((terminated($head, space1), $first $(, $(preceded(isep, $tail)),* )? ))
    };
}

fn un_alu(i: &str) -> Res<UnAlu> {
    alt((
        value(UnAlu::Neg, tag("neg")),
        value(UnAlu::Le, tag("le")),
        value(UnAlu::Be, tag("be")),
    ))(i)
}

fn bin_alu(i: &str) -> Res<BinAlu> {
    alt((
        value(BinAlu::Mov, tag("mov")),
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
    ))(i)
}

fn unary(i: &str) -> Res<Instr> {
    instr!(pair(un_alu, alu_size), reg)
        .map(|((op, size), reg)| Instr::Unary(size, op, reg))
        .parse(i)
}

fn binary(i: &str) -> Res<Instr> {
    instr!(pair(bin_alu, alu_size), reg, reg_imm)
        .map(|((op, size), reg, reg_imm)| Instr::Binary(size, op, reg, reg_imm))
        .parse(i)
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
    let inner = map(
        tuple((reg, space0, opt(offset), space0)),
        |(reg, _, offset, _)| MemRef(reg, offset),
    );
    delimited(terminated(char('['), space0), inner, char(']'))(i)
}

fn load(i: &str) -> Res<Instr> {
    map(
        instr!(preceded(tag("ldx"), mem_size), reg, mem_ref),
        |(size, reg, mem_ref)| Instr::Load(size, reg, mem_ref),
    )(i)
}

fn store(i: &str) -> Res<Instr> {
    instr!(
        preceded(alt((tag("stx"), tag("st"))), mem_size),
        mem_ref,
        reg_imm
    )
    .map(|(size, mref, reg_imm)| Instr::Store(size, mref, reg_imm))
    .parse(i)
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
        instr!(
            preceded(char('j'), cc),
            reg,
            reg_imm,
            ident.map(|id| id.to_owned())
        ),
        |(cc, lhs, rhs, target)| Instr::Jcc(cc, lhs, rhs, target),
    )(i)
}

fn instr(i: &str) -> Res<Instr> {
    let jmp = map(
        preceded(pair(tag("ja"), space1), ident.map(|id| id.to_owned())),
        Instr::Jmp,
    );
    let call = map(preceded(pair(tag("call"), space1), imm), Instr::Call);
    let load_imm = map(instr!(tag("lddw"), reg, imm), |(_, reg, imm)| {
        Instr::LoadImm(reg, imm)
    });
    let exit = value(Instr::Exit, tag("exit"));
    // Missing: LoadMapFd
    alt((unary, binary, load, load_imm, store, jcc, jmp, call, exit))(i)
}

// Assertion parsing
fn parens<'a, T>(p: impl FnMut(&'a str) -> Res<'a, T>) -> impl FnMut(&'a str) -> Res<'a, T> {
    delimited(
        terminated(char('('), space0),
        terminated(p, space0),
        terminated(char(')'), space0),
    )
}

fn reg_ident(i: &str) -> Res<Ident> {
    map(reg, |r| format!("r{}", r.get()))(i)
}

fn expr(i: &str) -> Res<Expr> {
    let unary = tuple((un_alu, parens(expr))).map(|(op, inner)| Expr::Unary(op, Box::new(inner)));
    let binary = tuple((
        bin_alu,
        parens(tuple((expr, space0, char(','), space0, expr))),
    ))
    .map(|(op, (a, _, _, _, b))| Expr::Binary(op, Box::new((a, b))));

    alt((reg_ident.map(Expr::Var), imm.map(Expr::Val), unary, binary))(i)
}

fn formula(i: &str) -> Res<Formula> {
    let parenthesized = parens(formula);
    let val = alt((
        value(Formula::Val(true), tag("true")),
        value(Formula::Val(false), tag("false")),
    ));
    let not = preceded(tag("not"), parens(formula)).map(|inner| Formula::Not(Box::new(inner)));

    let bin_op = alt((
        value(FBinOp::And, tag("/\\")),
        value(FBinOp::Or, tag("\\/")),
        value(FBinOp::Implies, tag("->")),
        value(FBinOp::Iff, tag("<->")),
        value(FBinOp::AndAsym, tag("&&")),
    ));

    let binary = tuple((
        bin_op,
        parens(tuple((formula, space0, char(','), space0, formula))),
    ))
    .map(|(op, (a, _, _, _, b))| Formula::Bin(op, Box::new((a, b))));

    let quantifier = alt((
        value(QType::Forall, tag("forall")),
        value(QType::Exists, tag("exists")),
    ));

    let quant = tuple((quantifier, space1, ident, char('.'), space0, formula)).map(
        |(quantifier, _, name, _, _, inner)| {
            Formula::Quant(quantifier, name.to_owned(), Box::new(inner))
        },
    );

    let rel_op = alt((
        value(Cc::Eq, tag("=")),
        value(Cc::Ne, tag("<>")),
        value(Cc::Gt, tag(">")),
        value(Cc::Lt, tag("<")),
        value(Cc::Ge, tag(">=")),
        value(Cc::Le, tag("<=")),
        // TODO: signed comparisons and 'set'
    ));

    let rel = tuple((expr, space0, rel_op, space0, expr))
        .map(|(e1, _, op, _, e2)| Formula::Rel(op, e1, e2));

    alt((parenthesized, val, not, binary, quant, rel))(i)
}

fn assertion(i: &str) -> Res<Formula> {
    preceded(tuple((tag(";#"), space0, tag("assert"), space0)), formula)(i)
}

// Structural parsing

fn line_sep(i: &str) -> Res<()> {
    value(
        (),
        many1(tuple((
            space0,
            opt(pair(
                terminated(char(';'), peek(satisfy(|c| c != '#'))),
                many0(satisfy(|c| c != '\n')),
            )),
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
    alt((
        label.map(Line::Label),
        assertion.map(Line::Assert),
        instr.map(Line::Instr),
    ))(i)
}

pub fn module(i: &str) -> Res<Module> {
    delimited(
        opt(line_sep),
        separated_list0(line_sep, line),
        pair(opt(line_sep), eof),
    )(i)
}
