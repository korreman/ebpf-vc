//! Parsing of eBPF assembly.

use nom::{
    branch::alt, bytes::complete::tag, character::complete::*, combinator::*, multi::*,
    sequence::*, IResult, Parser,
};

use crate::ast::*;

#[cfg(test)]
#[rustfmt::skip]
mod tests;

// TODO: Improve the whitespace story.

type Res<'a, O> = IResult<&'a str, O>;

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
    map_opt(
        preceded(
            char('r'),
            alt((
                tag("10"),
                tag("0"),
                tag("1"),
                tag("2"),
                tag("3"),
                tag("4"),
                tag("5"),
                tag("6"),
                tag("7"),
                tag("8"),
                tag("9"),
            )),
        ),
        |num: &str| Some(Reg::new(num.parse::<u8>().ok()?))?,
    )(i)
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
        preceded(pair(char('+'), space0), imm),
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

fn unary(i: &str) -> Res<Stmt> {
    instr!(pair(un_alu, alu_size), reg)
        .map(|((op, size), reg)| Stmt::Unary(size, op, reg))
        .parse(i)
}

fn binary(i: &str) -> Res<Stmt> {
    instr!(pair(bin_alu, alu_size), reg, reg_imm)
        .map(|((op, size), reg, reg_imm)| Stmt::Binary(size, op, reg, reg_imm))
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
        |(reg, _, offset, _)| MemRef(reg, offset.unwrap_or(0)),
    );
    delimited(terminated(char('['), space0), inner, char(']'))(i)
}

fn load(i: &str) -> Res<Stmt> {
    map(
        instr!(preceded(tag("ldx"), mem_size), reg, mem_ref),
        |(size, reg, mem_ref)| Stmt::Load(size, reg, mem_ref),
    )(i)
}

fn store(i: &str) -> Res<Stmt> {
    instr!(
        preceded(alt((tag("stx"), tag("st"))), mem_size),
        mem_ref,
        reg_imm
    )
    .map(|(size, mref, reg_imm)| Stmt::Store(size, mref, reg_imm))
    .parse(i)
}

fn cont(i: &str) -> Res<Cont> {
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
    let jcc = map(
        instr!(
            preceded(char('j'), cc),
            reg,
            reg_imm,
            ident.map(|id| id.to_owned())
        ),
        |(cc, lhs, rhs, target)| Cont::Jcc(cc, lhs, rhs, target),
    );

    let jmp = map(
        preceded(
            pair(alt((tag("ja"), tag("jmp"))), space1),
            ident.map(|id| id.to_owned()),
        ),
        Cont::Jmp,
    );
    let exit = value(Cont::Exit, tag("exit"));

    alt((exit, jmp, jcc))(i)
}

fn stmt(i: &str) -> Res<Stmt> {
    let call = map(preceded(pair(tag("call"), space1), imm), Stmt::Call);
    let load_imm = map(instr!(tag("lddw"), reg, imm), |(_, reg, imm)| {
        Stmt::LoadImm(reg, imm)
    });
    // Missing: LoadMapFd
    alt((unary, binary, load, load_imm, store, call))(i)
}

// Assertion parsing
fn parens<'a, T>(p: impl FnMut(&'a str) -> Res<'a, T>) -> impl FnMut(&'a str) -> Res<'a, T> {
    delimited(
        terminated(char('('), space0),
        terminated(p, space0),
        terminated(char(')'), space0),
    )
}

fn expr(i: &str) -> Res<Expr> {
    let unary = tuple((un_alu, parens(expr))).map(|(op, inner)| Expr::Unary(op, Box::new(inner)));
    let binary = tuple((
        terminated(bin_alu, space0),
        parens(tuple((expr, space0, char(','), space0, expr))),
    ))
    .map(|(op, (a, _, _, _, b))| Expr::Binary(op, Box::new((a, b))));

    alt((
        binary,
        unary,
        imm.map(Expr::Val),
        ident.map(|id| Expr::Var(id.to_owned())),
    ))(i)
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
        value(Cc::Ge, tag(">=")),
        value(Cc::Le, tag("<=")),
        value(Cc::Lt, tag("<")),
        value(Cc::Gt, tag(">")),
        // TODO: signed comparisons and 'set'
    ));

    let rel = tuple((expr, space0, rel_op, space0, expr))
        .map(|(e1, _, op, _, e2)| Formula::Rel(op, e1, e2));

    let is_buffer = tuple((
        tag("is_buffer"),
        space0,
        parens(tuple((ident, char(','), space0, expr, space0))),
    ))
    .map(|(_, _, (id, _, _, e, _))| Formula::IsBuffer(id.to_owned(), e));
    alt((parenthesized, val, not, binary, quant, rel, is_buffer))(i)
}

fn formula_line(i: &str) -> Res<Logic> {
    preceded(
        pair(tag(";#"), space0),
        alt((
            preceded(pair(tag("assert"), space0), map(formula, Logic::Assert)),
            preceded(pair(tag("req"), space0), map(formula, Logic::Require)),
        )),
    )(i)
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
        formula_line.map(Line::Logic),
        stmt.map(Line::Stmt),
        cont.map(Line::Cont),
    ))(i)
}

pub fn module(i: &str) -> Res<Module> {
    let requirement = preceded(tuple((tag(";#"), space0, tag("requires"), space0)), formula);
    let ensurance = preceded(tuple((tag(";#"), space0, tag("ensures"), space0)), formula);
    let components = tuple((
        many0(terminated(requirement, line_sep)),
        many0(terminated(ensurance, line_sep)),
        preceded(space0, separated_list0(line_sep, line)),
    ));
    delimited(
        opt(line_sep),
        map(components, |(rs, es, ls)| Module {
            lines: ls,
            requires: rs,
            ensures: es,
        }),
        pair(opt(line_sep), eof),
    )(i)
}
