use std::fmt::Debug;

use super::*;
use nom::{combinator::eof, error::ParseError, sequence::terminated};

fn accepts<'a, T, E: ParseError<&'a str>>(parser: impl Parser<&'a str, T, E>, input: &'a str) {
    if terminated(parser, eof).parse(input).is_err() {
        panic!("rejects: {input:?}")
    }
}

fn rejects<'a, T, E: ParseError<&'a str>>(parser: impl Parser<&'a str, T, E>, input: &'a str) {
    if terminated(parser, eof).parse(input).is_ok() {
        panic!("accepts: {input:?}")
    }
}

fn parses<'a, T: Eq + Debug, E: ParseError<&'a str>>(
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
        Err(_) => panic!("rejected: {input:?}"),
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

    rejects(num, "");
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

    rejects(ident, "");
    rejects(ident, "_");
    rejects(ident, "__");
    rejects(ident, "________");
    rejects(ident, "123");
    rejects(ident, "123abc");
    rejects(ident, "_123");
    rejects(ident, "_abc ");
    rejects(ident, " _abc");
    for c in " `~!@#$%^&*[]+-=()\\{}|;':\",./<>?".chars() {
        rejects(ident, format!("hello{c}world").as_str());
    }
}

fn instructions() {
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
