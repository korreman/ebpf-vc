mod ast;
mod logic;
mod vc;

fn test() {
    let i = ast::Instr::Unary(
        ast::WordSize::B64,
        ast::UnAlu::Neg,
        ast::Reg::new(9).unwrap(),
    );
}
