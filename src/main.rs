use argh::FromArgs;

use std::{ffi::OsString, process::ExitCode};

use ebpf_vc::{
    cfg::{Cfg, ConvertErr},
    formula::FormulaBuilder,
    parse::module,
    vc::vc,
    whyml::Conditions,
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
    let preprocess_res: Result<Cfg, ConvertErr> = Cfg::create(ast, &mut f);
    let processed_ast = match preprocess_res {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    //eprintln!("{processed_ast:#?}\n");

    let vc_res = vc(processed_ast, &mut f);
    println!("{}", Conditions(vc_res));
    ExitCode::SUCCESS
}
