use super::*;
use nom::{combinator::eof, sequence::terminated};

// TODO: better error messages
macro_rules! accepts {
    ( $parser:ident, $input:expr ) => {
        parses!($parser, $input, $input);
    };
}

macro_rules! parses {
    ( $parser:ident, $input:expr, $output:expr) => {
        assert_eq!(
            nom::sequence::terminated($parser, nom::combinator::eof)($input),
            Ok(("", $output))
        );
    };
}

macro_rules! rejects {
    ( $parser:ident, $input:expr ) => {
        assert!(
            match nom::sequence::terminated($parser, nom::combinator::eof)($input) {
                Err(_) => true,
                _ => false,
            },
            "accepted \"{}\"",
            $input,
        );
    };
}

#[test]
fn numbers() {
    // decimal
    parses!(num, "0", 0);
    parses!(num, "1", 1);
    parses!(num, "9", 9);
    parses!(num, "39824", 39824);
    parses!(num, "1234567890", 1234567890);
    parses!(num, "348_922_30_875_", 34892230875);

    rejects!(num, "_123");
    rejects!(num, "123a");
    rejects!(num, "123a456");
    rejects!(num, "a123");
    rejects!(num, "");

    // hexadecimal
    parses!(num, "0x0", 0);
    parses!(num, "0x1", 1);
    parses!(num, "0xf", 0xf);
    parses!(num, "0x1234567890abcdef", 0x1234567890abcdef);
    parses!(num, "0xdeadbeef", 0xdeadbeef);
    parses!(num, "0xdE_aD_bE_eF_", 0xdeadbeef);

    rejects!(num, "0x_123");
    rejects!(num, "_0x123");
    rejects!(num, "x123");
    rejects!(num, "0x");
    rejects!(num, "0xx");
    rejects!(num, "deadbeef");

    // binary
    parses!(num, "0b0", 0);
    parses!(num, "0b1", 1);
    parses!(num, "0b11", 3);
    parses!(num, "0b101010", 42);
    parses!(num, "0b11010010", 0b11010010);
    parses!(num, "0b1_1_010_010_", 0b11010010);

    parses!(num, "1_1_010_010_", 11010010);
    rejects!(num, "0b123");
    rejects!(num, "0bb");
    rejects!(num, "0b_101");
    rejects!(num, "_0b101");
    rejects!(num, "b101");
    rejects!(num, "0b");
}

#[test]
fn identifiers() {
    accepts!(ident, "identifier");
    accepts!(ident, "identifier_again");
    accepts!(ident, "abcdefghijklmnopqrstuvwxyzæøå");
    accepts!(ident, "_hello_world_");

    rejects!(ident, "_");
    rejects!(ident, "__");
    rejects!(ident, "________");
    rejects!(ident, "________");
}
