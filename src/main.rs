use argh::FromArgs;

use std::{ffi::OsString, process::ExitCode, str::FromStr};

use ebpf_vc::{
    cfg::{Cfg, ConvertErr},
    formula::FormulaBuilder,
    parse::module,
    vc::vc,
    whyml,
};

#[derive(FromArgs)]
/// A verification condition generator for eBPF.
struct EbpfVc {
    /// input to generate conditions for
    #[argh(positional)]
    file: OsString,
    /// proof obligation format (default is WhyML)
    #[argh(option, default = "OutputFmt::WhyML")]
    format: OutputFmt,
}

enum OutputFmt {
    WhyML,
    CVC5,
}

impl FromStr for OutputFmt {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fmt = match s.to_ascii_lowercase().as_str() {
            "whyml" => Self::WhyML,
            "cvc5" => Self::CVC5,
            _ => return Err("unknown output format"),
        };
        Ok(fmt)
    }
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
        Err(e) => {
            eprintln!("error: failed to parse module - {e}");
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
    match opts.format {
        OutputFmt::WhyML => println!("{}", whyml::Conditions(vc_res)),
        OutputFmt::CVC5 => eprintln!("Architecture currently cannot support both formats"), //println!("{}", Conditions(vc_res)),
    }
    ExitCode::SUCCESS
}
