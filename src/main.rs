use std::{ffi::OsString, process::ExitCode};

use argh::FromArgs;

use ebpf_vc::{
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
    //println!("{ast:#?}\n");

    let preprocess_res: Result<Module, ConvertErr> = ast.try_into();
    let processed_ast = match preprocess_res {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    //println!("{processed_ast:#?}\n");

    let vc_res = vc(processed_ast);
    match vc_res {
        Ok(res) => {
            println!("use mach.int.UInt64\nuse int.ComputerDivision\n");
            for (i, f) in res.iter().enumerate() {
                println!("goal G{i}: {f}");
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}
