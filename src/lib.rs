mod ast;
mod logic;

fn test() {
    let i = ast::Instr::Unary(
        ast::WordSize::B64,
        ast::UnAlu::Neg,
        ast::Reg::new(9).unwrap(),
    );
}
