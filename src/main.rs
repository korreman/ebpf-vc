use std::{ffi::OsString, process::ExitCode};

use bpaf::Bpaf;
use ebpf_vc::{
    parse::module,
    vc::{ast::Module, vc, ConvertErr},
};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Command {
    /// input file to generate conditions for
    #[bpaf(positional("file"))]
    input: OsString,
}

fn main() -> ExitCode {
    let opts = command().run();

    let file = std::fs::read_to_string(opts.input);
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

    let preprocess_res: Result<Module, ConvertErr> = ast.try_into();
    let processed_ast = match preprocess_res {
        Ok(p) => p,
        Err(e) => {
            let estr = match e {
                ConvertErr::Invalid => "Invalid AST",
                ConvertErr::Unsupported => "Use of unsupported feature",
                ConvertErr::Internal => "Internal error in pre-processing",
            };
            eprintln!("error: {estr}");
            return ExitCode::FAILURE;
        }
    };

    let vc_res = vc(processed_ast);
    match vc_res {
        Some(res) => println!("{res:?}"),
        None => {
            eprintln!("error: condition generation failed");
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}
