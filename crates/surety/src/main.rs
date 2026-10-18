use std::env;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = "\
Usage: surety [--help | --version]
Surety 0.0.1 is a placeholder release that reserves the crate name. No
language features are implemented: --version prints the version, --help
prints this text, and any other invocation prints \"not yet implemented\"
to stderr and exits with status 2. Follow along at
https://github.com/surety-lang.";

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [flag] if flag == "--version" => {
            println!("surety {VERSION}");
            ExitCode::SUCCESS
        }
        [flag] if flag == "--help" => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("surety {VERSION}: not yet implemented");
            ExitCode::from(2)
        }
    }
}
