use std::fmt::Debug;


use super::*;
use nom::{combinator::eof, error::ParseError, sequence::terminated};

fn accepts<'a, T, E: Debug + ParseError<&'a str>>
    (parser: impl Parser<&'a str, T, E>, input: &'a str) {
    if let Err(e) = terminated(parser, eof).parse(input) {
        if input.len() > 32 && input.contains('\n') {
            panic!("rejects:\n\n{input}\n\nerror: {e:?}")
        } else {
            panic!("rejects: {input:?}\nerror: {e:?}")
        }
    }
}

fn rejects<'a, T, E: ParseError<&'a str>>(parser: impl Parser<&'a str, T, E>, input: &'a str) {
    if terminated(parser, eof).parse(input).is_ok() {
        panic!("accepts: {input:?}")
    }
}

fn parses<'a, T: Eq + Debug, E: Debug + ParseError<&'a str>>(
    parser: impl Parser<&'a str, T, E>,
    input: &'a str,
    expected: T,
) {
    match terminated(parser, eof).parse(input) {
        Ok((_, res)) => {
            if res != expected {
                panic!("\nexpected: {expected:?}\n  actual: {res:?}");
            }
        }
        Err(e) => panic!("rejects: {input:?}\nerror: {e:?}"),
    }
}

#[test]
fn numbers() {
    // decimal
    parses(num, "0", 0);
    parses(num, "1", 1);
    parses(num, "9", 9);
    parses(num, "009", 9);
    parses(num, "39824", 39824);
    parses(num, "1234567890", 1234567890);
    parses(num, "348_922_30_875_", 34892230875);

    rejects(num, "_123");
    rejects(num, "123a");
    rejects(num, "123a456");
    rejects(num, "123 456");
    rejects(num, "-123"); // NOTE: num doesn't itself parse signs
    rejects(num, "- 123");

    // hexadecimal
    parses(num, "0x0", 0);
    parses(num, "0x1", 1);
    parses(num, "0xf", 0xf);
    parses(num, "0x1234567890abcdef", 0x1234567890abcdef);
    parses(num, "0xdeadbeef", 0xdeadbeef);
    parses(num, "0xdE_aD_bE_eF_", 0xdeadbeef);

    rejects(num, "0x_123");
    rejects(num, "_0x123");
    rejects(num, "x123");
    rejects(num, "0x");
    rejects(num, "0xx");
    rejects(num, "deadbeef");

    // binary
    parses(num, "0b0", 0);
    parses(num, "0b1", 1);
    parses(num, "0b11", 3);
    parses(num, "0b101010", 42);
    parses(num, "0b11010010", 0b11010010);
    parses(num, "0b1_1_010_010_", 0b11010010);

    parses(num, "1_1_010_010_", 11010010);
    rejects(num, "0b123");
    rejects(num, "0bb");
    rejects(num, "0b_101");
    rejects(num, "_0b101");
    rejects(num, "b101");
    rejects(num, "0b");
}

#[test]
fn identifiers() {
    accepts(ident, "identifier");
    accepts(ident, "identifier_again");
    accepts(ident, "sphinx_of_black_quartz_judge_my_vow");
    accepts(ident, "PACK_MY_BOX_WITH_FIVE_DOZEN_LIQOUR_JUGS");
    accepts(ident, "_hello_WORLD_");
    accepts(ident, "abc123");

    rejects(ident, "_");
    rejects(ident, "__");
    rejects(ident, "________");
    rejects(ident, "123");
    rejects(ident, "123abc");
    rejects(ident, "_123");
    for c in " `~!@#$%^&*[]+-=()\\{}|;':\",./<>?".chars() {
        rejects(ident, format!("hello{c}world").as_str());
    }
}

#[test]
fn instruction_separators() {
    accepts(isep, " ");
    accepts(isep, " ,");
    accepts(isep, " , ");
    accepts(isep, "\t , ");
    accepts(isep, "\t , \t");
    accepts(isep, "\t\t , \t\t");
    accepts(isep, "  \t\t    ,    \t\t  ");

    rejects(isep, "");
    rejects(isep, ",,");
    rejects(isep, "\n");
    rejects(isep, " \n ");
    rejects(isep, ", \n ");
    rejects(isep, " \n ,");
}

#[test]
fn registers() {
    parses(reg, "r0", Reg::new(0).unwrap());
    parses(reg, "r1", Reg::new(1).unwrap());
    parses(reg, "r2", Reg::new(2).unwrap());
    parses(reg, "r3", Reg::new(3).unwrap());
    parses(reg, "r4", Reg::new(4).unwrap());
    parses(reg, "r5", Reg::new(5).unwrap());
    parses(reg, "r6", Reg::new(6).unwrap());
    parses(reg, "r7", Reg::new(7).unwrap());
    parses(reg, "r8", Reg::new(8).unwrap());
    parses(reg, "r9", Reg::new(9).unwrap());

    rejects(reg, "r");
    rejects(reg, "rcx");
    rejects(reg, "r10");
    rejects(reg, "r11");
    rejects(reg, "%r11");
}

#[test]
fn immediates() {
    parses(imm, "123", 123);

    parses(imm, "+123", 123);
    parses(imm, "+0x38fa", 0x38fa);
    parses(imm, "+0b101010", 42);
    parses(imm, "-123", -123);
    parses(imm, "-0x38fa", -0x38fa);
    parses(imm, "-0b101010", -42);

    parses(imm, "+ \t 123", 123);
    parses(imm, "+ \t 0x38fa", 0x38fa);
    parses(imm, "+ \t 0b101010", 42);
    parses(imm, "- \t 123", -123);
    parses(imm, "- \t 0x38fa", -0x38fa);
    parses(imm, "- \t 0b101010", -42);
}

#[test]
fn register_or_immediates() {
    parses(reg_imm, "r0", RegImm::Reg(Reg::new(0).unwrap()));
    parses(reg_imm, "r1", RegImm::Reg(Reg::new(1).unwrap()));
    parses(reg_imm, "r8", RegImm::Reg(Reg::new(8).unwrap()));
    parses(reg_imm, "r9", RegImm::Reg(Reg::new(9).unwrap()));

    parses(reg_imm, "123", RegImm::Imm(123));
    parses(reg_imm, "-123", RegImm::Imm(-123));
    parses(reg_imm, "0xde_aDBe_Ef", RegImm::Imm(0xdeadbeef));
    parses(reg_imm, "-0xde_aDBe_Ef", RegImm::Imm(-0xdeadbeef));
    parses(reg_imm, "-0b1001", RegImm::Imm(-0b1001));

    rejects(reg_imm, "-r0");
    rejects(reg_imm, "r0 ");
    rejects(reg_imm, "0 ");
    rejects(reg_imm, "label ");
}

#[test]
fn memory_references() {
    parses(mem_ref, "[r0]", MemRef(Reg::R0, None));
    parses(mem_ref, "[ r0 ]", MemRef(Reg::R0, None));
    parses(mem_ref, "[ r9 + 24 ]", MemRef(Reg::R9, Some(24)));
    parses(mem_ref, "[ r5 -\t 3 ]", MemRef(Reg::R5, Some(-3)));
    parses(mem_ref, "[ r5 -\t 0b1001001 ]", MemRef(Reg::R5, Some(-0b1001001)));
    parses(mem_ref, "[r2+0xdeadbeef]", MemRef(Reg::R2, Some(0xdeadbeef)));

    rejects(mem_ref, "r0");
    rejects(mem_ref, "[r0");
    rejects(mem_ref, "r0]");
    rejects(mem_ref, "[[r0]");
    rejects(mem_ref, "[r0]]");
    rejects(mem_ref, "[[r0]]");
    rejects(mem_ref, "[]");
    rejects(mem_ref, "[r4 10]");
    rejects(mem_ref, "[10 + r4]");
}

#[test]
fn unary_instrs() {
    parses(stmt, "neg r0", Stmt::Unary(WordSize::B64, UnAlu::Neg, Reg::R0));
    parses(stmt, "le r3", Stmt::Unary(WordSize::B64, UnAlu::Le, Reg::R3));
    parses(stmt, "be r8", Stmt::Unary(WordSize::B64, UnAlu::Be, Reg::R8));

    parses(stmt, "neg32 r0", Stmt::Unary(WordSize::B32, UnAlu::Neg, Reg::R0));
    parses(stmt, "le32 r3", Stmt::Unary(WordSize::B32, UnAlu::Le, Reg::R3));
    parses(stmt, "be32 r8", Stmt::Unary(WordSize::B32, UnAlu::Be, Reg::R8));

    rejects(stmt, "neg r0,");
    rejects(stmt, "neg32, r0");
    rejects(stmt, "neg16 r0");
    rejects(stmt, "neg8 r0");
    rejects(stmt, "neg r0 r1");
    rejects(stmt, "be r0 r1");
    rejects(stmt, "le r0 r1");
}

#[test]
fn binary_instrs() {
    parses(stmt, "mov r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "add r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Add, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "sub r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Sub, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "mul r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Mul, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "div r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Div, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "mod r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Mod, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "and r1 r2", Stmt::Binary(WordSize::B64, BinAlu::And, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "or r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Or, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "xor r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Xor, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "lsh r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Lsh, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "rsh r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Rsh, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "arsh r1 r2", Stmt::Binary(WordSize::B64, BinAlu::Arsh, Reg::R1, RegImm::Reg(Reg::R2)));

    parses(stmt, "mov r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "add r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Add, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "sub r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Sub, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "mul r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Mul, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "div r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Div, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "mod r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Mod, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "and r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::And, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "or r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Or, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "xor r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Xor, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "lsh r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Lsh, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "rsh r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Rsh, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "arsh r1 0xfaef", Stmt::Binary(WordSize::B64, BinAlu::Arsh, Reg::R1, RegImm::Imm(0xfaef)));

    parses(stmt, "mov32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Mov, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "add32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Add, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "sub32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Sub, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "mul32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Mul, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "div32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Div, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "mod32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Mod, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "and32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::And, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "or32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Or, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "xor32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Xor, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "lsh32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Lsh, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "rsh32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Rsh, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "arsh32 r1 r2", Stmt::Binary(WordSize::B32, BinAlu::Arsh, Reg::R1, RegImm::Reg(Reg::R2)));

    parses(stmt, "mov32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Mov, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "add32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Add, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "sub32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Sub, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "mul32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Mul, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "div32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Div, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "mod32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Mod, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "and32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::And, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "or32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Or, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "xor32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Xor, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "lsh32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Lsh, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "rsh32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Rsh, Reg::R1, RegImm::Imm(0xfaef)));
    parses(stmt, "arsh32 r1 0xfaef", Stmt::Binary(WordSize::B32, BinAlu::Arsh, Reg::R1, RegImm::Imm(0xfaef)));

    parses(stmt, "mov32 r1, - 0b1001", Stmt::Binary(WordSize::B32, BinAlu::Mov, Reg::R1, RegImm::Imm(-0b1001)));
    parses(stmt, "or32 r1, r2", Stmt::Binary(WordSize::B32, BinAlu::Or, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "div32 r1,+0b1001", Stmt::Binary(WordSize::B32, BinAlu::Div, Reg::R1, RegImm::Imm(0b1001)));
    parses(stmt, "mul32 r1,r2", Stmt::Binary(WordSize::B32, BinAlu::Mul, Reg::R1, RegImm::Reg(Reg::R2)));
    parses(stmt, "add64 r1, r2", Stmt::Binary(WordSize::B64, BinAlu::Add, Reg::R1, RegImm::Reg(Reg::R2)));

    rejects(stmt, "mov");
    rejects(stmt, "mov,");
    rejects(stmt, "mov r1");
    rejects(stmt, "mov r1,");
    rejects(stmt, "mov32, r1, r2");
    rejects(stmt, "mov32, r1, r2");
    rejects(stmt, "mov32 r1,, 0xfaef");
    rejects(stmt, "mov32r1 0xfaef");
    rejects(stmt, "movr1 0xfaef");
    rejects(stmt, "mov r1r2");
    rejects(stmt, "mov 32 r1 r2");
    rejects(stmt, "mov 0 r2");
}

#[test]
fn load_instructions() {
    parses(stmt, "ldxb r0  [r1]", Stmt::Load(WordSize::B8, Reg::R0, MemRef(Reg::R1, None)));
    parses(stmt, "ldxh r0  [r1]", Stmt::Load(WordSize::B16, Reg::R0, MemRef(Reg::R1, None)));
    parses(stmt, "ldxw r0  [r1]", Stmt::Load(WordSize::B32, Reg::R0, MemRef(Reg::R1, None)));
    parses(stmt, "ldxdw r0 [r1]", Stmt::Load(WordSize::B64, Reg::R0, MemRef(Reg::R1, None)));
    parses(stmt, "lddw r0, 123", Stmt::LoadImm(Reg::R0, 123));

    rejects(stmt, "ld r0 [r1]");
    rejects(stmt, "ldx r0 [r1]");
}

#[test]
fn store_instructions() {
    parses(stmt, "stb  [r0] 123", Stmt::Store(WordSize::B8, MemRef(Reg::R0, None), RegImm::Imm(123)));
    parses(stmt, "sth  [r0] 123", Stmt::Store(WordSize::B16, MemRef(Reg::R0, None), RegImm::Imm(123)));
    parses(stmt, "stw  [r0] 123", Stmt::Store(WordSize::B32, MemRef(Reg::R0, None), RegImm::Imm(123)));
    parses(stmt, "stdw [r0] 123", Stmt::Store(WordSize::B64, MemRef(Reg::R0, None), RegImm::Imm(123)));

    parses(stmt, "stxb  [r0] r1", Stmt::Store(WordSize::B8, MemRef(Reg::R0, None), RegImm::Reg(Reg::R1)));
    parses(stmt, "stxh  [r0] r1", Stmt::Store(WordSize::B16, MemRef(Reg::R0, None), RegImm::Reg(Reg::R1)));
    parses(stmt, "stxw  [r0] r1", Stmt::Store(WordSize::B32, MemRef(Reg::R0, None), RegImm::Reg(Reg::R1)));
    parses(stmt, "stxdw [r0] r1", Stmt::Store(WordSize::B64, MemRef(Reg::R0, None), RegImm::Reg(Reg::R1)));

    rejects(stmt, "st [r0] r1");
    rejects(stmt, "stx [r0] r1");
}

#[test]
fn jump_instructions() {
    parses(stmt, "ja label", Stmt::Jmp("label".to_owned()));
    parses(stmt, "jeq r0 0 l", Stmt::Jcc(Cc::Eq, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jeq r0 r1 l", Stmt::Jcc(Cc::Eq, Reg::R0, RegImm::Reg(Reg::R1), "l".to_owned()));

    parses(stmt, "jgt r0 0 l", Stmt::Jcc(Cc::Gt, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jge r0 0 l", Stmt::Jcc(Cc::Ge, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jlt r0 0 l", Stmt::Jcc(Cc::Lt, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jle r0 0 l", Stmt::Jcc(Cc::Le, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jset r0 0 l", Stmt::Jcc(Cc::Set, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jne r0 0 l", Stmt::Jcc(Cc::Ne, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jsgt r0 0 l", Stmt::Jcc(Cc::Sgt, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jsge r0 0 l", Stmt::Jcc(Cc::Sge, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jslt r0 0 l", Stmt::Jcc(Cc::Slt, Reg::R0, RegImm::Imm(0), "l".to_owned()));
    parses(stmt, "jsle r0 0 l", Stmt::Jcc(Cc::Sle, Reg::R0, RegImm::Imm(0), "l".to_owned()));

    rejects(stmt, "jeq 0 r1 l");
}

#[test]
fn line_separators() {
    accepts(line_sep, "\n");
    accepts(line_sep, "    \n");
    accepts(line_sep, "\t    \n");
    accepts(line_sep, "    \n   ");
    accepts(line_sep, ";\n");
    accepts(line_sep, "   ;\n");
    accepts(line_sep, "   ;\n   ");
    accepts(line_sep, ";comment\n");
    accepts(line_sep, "   ; a comment \n   ");
    accepts(line_sep, "   ;; ; comments can be unicode: æØ§ŒfïÐ§ÓÍ \n   ");
    accepts(line_sep, "\n\n\n");
    accepts(line_sep, "\n;\n\n");
    accepts(line_sep, "; comment \n\n ;comment   \n");

    rejects(line_sep, "");
    rejects(line_sep, ";#");
    rejects(line_sep, ";#some comment");
    rejects(line_sep, ";# some comment");
    rejects(line_sep, ";# assert x <> y");
    rejects(line_sep, " ");
    rejects(line_sep, "a\n");
    rejects(line_sep, "a \n");
    rejects(line_sep, "not a comment \n");
    rejects(line_sep, "\n ; comments have to end with a newline");
    rejects(line_sep, "\n ; \n; \n;");
}

#[test]
fn labels() {
    let positives = [
        ("l:", "l"),
        ("l :", "l"),
        ("LABEL :", "LABEL"),
        ("_XYZ123_:", "_XYZ123_"),
        ("_XYZ123_  \t  :", "_XYZ123_"),
    ];

    for (input, expect) in positives {
        parses(label, input, expect.to_owned());
    }

    rejects(label, ":");
    rejects(label, "some_identifier");
    rejects(label, "some identifiers");
}

#[test]
fn formulas() {
    let f = crate::formula::FormulaBuilder::new();
    parses(formula, "true", f.top());
    parses(formula, "false", f.bot());

    let x = f.var_ident("x".to_owned());
    let y = f.var_ident("y".to_owned());
    let z = f.var_ident("z".to_owned());
    parses(formula, "x = y", f.rel(Cc::Eq, x.clone(), y.clone()));
    parses(formula, "x <> y", f.rel(Cc::Ne, x.clone(), y.clone()));
    parses(formula, "x > y", f.rel(Cc::Gt, x.clone(), y.clone()));
    parses(formula, "x >= y", f.rel(Cc::Ge, x.clone(), y.clone()));
    parses(formula, "x < y", f.rel(Cc::Lt, x.clone(), y.clone()));
    parses(formula, "x <= y", f.rel(Cc::Le, x.clone(), y.clone()));

    parses(
        formula, "x <= sub(y, z)",
        f.rel(Cc::Le, x.clone(), f.binop(BinAlu::Sub, y.clone(), z.clone()))
    );
    parses(
        formula, "sub(x, y) >= add(mov(neg(x), z), y)",
        f.rel(
            Cc::Ge,
            f.binop(BinAlu::Sub, x.clone(), y.clone()),
            f.binop(BinAlu::Add, f.binop(BinAlu::Mov, f.unop(UnAlu::Neg, x.clone()), z.clone()), y.clone())
        )
    );
}

#[test]
fn assertions() {
    let f = crate::formulaBuilder::new();
    parses(assertion, ";# assert true", f.top());
    parses(assertion, ";# assert false", f.bot());

    let x = f.var_ident("x".to_owned());
    let y = f.var_ident("y".to_owned());
    parses(assertion, ";# assert x <> y", f.rel(Cc::Ne, x, y));
}

#[test]
fn gcd() {
    parses(module, include_str!("../../test_asm/exit.asm"), vec![Line::Stmtit)]);
    accepts(module, include_str!("../../test_asm/gcd.asm"));
    parses(
        module,
        "
            ;; Solution to day 1, part 1 of Advent of Code 2022.
            ;; The requires an
            ;; r0 - return value, max elf
            ;; r1 - input ptr
            ;; r2 - input size
            ;; r3 - index
            ;; r4 - load dst
            ;; r5 - number parsing accumulator
            ;; r6 - current elf

            mov r0 0
            mov r3 0
            mov r5 0
            mov r6 0
            outer: ;; loop
                jeq r3 r2 submit ;; submit final elf if end of input has been reached
                mov r4 r1 ;; load next byte
                add r4 r3
                ldxb r4 [r4]
                add r3 1
                jeq r4 10 submit ;; newline check

            inner: ;; loop. parses a number from a decimal string, terminated by newline
                mul r5 10
                add r5 r4
                sub r5 48
                mov r4 r1 ;; load next byte
                add r4 r3
                ldxb r4 [r4]
                add r3 1
                jne r4 10 inner ;; newline check

                add r6 r5
                mov r5 0
                ja outer

            submit: ;; finishes an elf. compare to current max and replace if better, reset elf to 0
                jgt r0 r6 skip
                mov r0 r6
            skip:
                mov r6 0
                jeq r3 r2 end
                ja outer

            end:
                exit
        ",
        vec![
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R0, RegImm::Imm(0))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R3, RegImm::Imm(0))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R5, RegImm::Imm(0))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R6, RegImm::Imm(0))),
            Line::Label("outer".to_owned()),
            Line::Instr(Stmt::Jcc(Cc::Eq, Reg::R3, RegImm::Reg(Reg::R2), "submit".to_owned())),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R4, RegImm::Reg(Reg::R1))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Add, Reg::R4, RegImm::Reg(Reg::R3))),
            Line::Instr(Stmt::Load(WordSize::B8, Reg::R4, MemRef(Reg::R4, None))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Add, Reg::R3, RegImm::Imm(1))),
            Line::Instr(Stmt::Jcc(Cc::Eq, Reg::R4, RegImm::Imm(10), "submit".to_owned())),
            Line::Label("inner".to_owned()),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mul, Reg::R5, RegImm::Imm(10))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Add, Reg::R5, RegImm::Reg(Reg::R4))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Sub, Reg::R5, RegImm::Imm(48))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R4, RegImm::Reg(Reg::R1))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Add, Reg::R4, RegImm::Reg(Reg::R3))),
            Line::Instr(Stmt::Load(WordSize::B8, Reg::R4, MemRef(Reg::R4, None))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Add, Reg::R3, RegImm::Imm(1))),
            Line::Instr(Stmt::Jcc(Cc::Ne, Reg::R4, RegImm::Imm(10), "inner".to_owned())),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Add, Reg::R6, RegImm::Reg(Reg::R5))),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R5, RegImm::Imm(0))),
            Line::Instr(Stmt::Jmp("outer".to_owned())),
            Line::Label("submit".to_owned()),
            Line::Instr(Stmt::Jcc(Cc::Gt, Reg::R0, RegImm::Reg(Reg::R6), "skip".to_owned())),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R0, RegImm::Reg(Reg::R6))),
            Line::Label("skip".to_owned()),
            Line::Instr(Stmt::Binary(WordSize::B64, BinAlu::Mov, Reg::R6, RegImm::Imm(0))),
            Line::Instr(Stmt::Jcc(Cc::Eq, Reg::R3, RegImm::Reg(Reg::R2), "end".to_owned())),
            Line::Instr(Stmt::Jmp("outer".to_owned())),
            Line::Label("end".to_owned()),
            Line::Instr(Stmt        ],
    );
}

#[test]
fn rejects_bad_module() {
    assert!(module("asdf").is_err());
}
