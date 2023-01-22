use std::{ffi::OsString, process::ExitCode};

use argh::FromArgs;

use ebpf_vc::{
    logic::FormulaBuilder,
    parse::module,
    vc::{ast::Module, vc, ConvertErr},
};

#[derive(FromArgs)]
/// A verification condition generator for eBPF.
struct EbpfVc {
    /// input to generate conditions for
    #[argh(positional)]
    file: OsString,
}

fn main() -> ExitCode {
    let opts: EbpfVc = argh::from_env();

    let file = std::fs::read_to_string(opts.file);
    let contents = match file {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let parsed_file = module(contents.as_str());
    let ast = match parsed_file {
        Ok((_, a)) => a,
        Err(_) => {
            eprintln!("error: Failed to parse module");
            return ExitCode::FAILURE;
        }
    };
    //eprintln!("{ast:#?}\n");

    let mut f = FormulaBuilder::new();
    let preprocess_res: Result<Module, ConvertErr> = ast.preprocess(&mut f);
    let processed_ast = match preprocess_res {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    //eprintln!("{processed_ast:#?}\n");

    let vc_res = vc(processed_ast, &mut f);
    println!(
        "use mach.int.UInt64\n\
        use int.Int\n\
        use int.ComputerDivision\n\
        predicate is_buffer (p: uint64) (s: uint64)\n"
    );
    for (i, f) in vc_res.iter().enumerate() {
        println!("goal G{i}: forall r0 r1 r2 r3 r4 r5 r6 r7 r8 r9 : uint64 . {f}");
    }
    ExitCode::SUCCESS
}
