use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use surety_syntax::extract::extract;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = "\
Usage:
  surety extract <markdown> <out-dir>   cut the obligations out of the authored markdown
  surety parse <file.sur>...            parse and check each file (siblings form the graph)
  surety hash <file.sur>                print the canonical form and its hash
  surety check <file.sur> [<perf.py>]   run the examples and properties against a performance
                                        (default: <file>.py next to the obligation)
  surety --version | --help";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let strs: Vec<&str> = args.iter().map(String::as_str).collect();
    match strs.as_slice() {
        ["--version"] => {
            println!("surety {VERSION}");
            ExitCode::SUCCESS
        }
        ["--help"] | [] => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        ["extract", md, out] => cmd_extract(Path::new(md), Path::new(out)),
        ["parse", files @ ..] if !files.is_empty() => cmd_parse(files),
        ["hash", file] => cmd_hash(Path::new(file)),
        ["check", file] => cmd_check(Path::new(file), None),
        ["check", file, perf] => cmd_check(Path::new(file), Some(Path::new(perf))),
        _ => {
            eprintln!("{HELP}");
            ExitCode::from(2)
        }
    }
}

fn cmd_extract(md: &Path, out: &Path) -> ExitCode {
    let text = match fs::read_to_string(md) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {}: {e}", md.display());
            return ExitCode::from(2);
        }
    };
    if let Err(e) = fs::create_dir_all(out) {
        eprintln!("cannot create {}: {e}", out.display());
        return ExitCode::from(2);
    }
    for ex in extract(&text) {
        let path: PathBuf = out.join(format!("{}.sur", ex.name));
        if let Err(e) = fs::write(&path, &ex.text) {
            eprintln!("cannot write {}: {e}", path.display());
            return ExitCode::from(2);
        }
        println!(
            "{}: lines {}.. -> {}",
            ex.name,
            ex.first_line,
            path.display()
        );
        for (line, dropped) in &ex.dropped {
            println!("  dropped {line}: {dropped}");
        }
    }
    ExitCode::SUCCESS
}

fn cmd_parse(files: &[&str]) -> ExitCode {
    let mut failed = false;
    for f in files {
        let path = Path::new(f);
        match surety::load_with_siblings(path) {
            Ok((graph, idx)) => match graph.check_file(idx) {
                Ok(()) => {
                    for f in graph.functions_in(idx) {
                        println!(
                            "{}: ok {} props={} examples={} abbrevs={} pins={}",
                            path.display(),
                            f.name,
                            f.properties.len(),
                            f.examples.len(),
                            f.abbrevs.len(),
                            graph.file(idx).pins.len()
                        );
                    }
                }
                Err(e) => {
                    println!("{}: {e}", path.display());
                    failed = true;
                }
            },
            Err(errs) => {
                for (p, e) in errs {
                    println!("{}: {e}", p.display());
                }
                failed = true;
            }
        }
    }
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn cmd_hash(file: &Path) -> ExitCode {
    let (graph, idx) = match surety::load_with_siblings(file) {
        Ok(x) => x,
        Err(errs) => {
            for (p, e) in errs {
                eprintln!("{}: {e}", p.display());
            }
            return ExitCode::from(1);
        }
    };
    if let Err(e) = graph.check_file(idx) {
        eprintln!("{}: {e}", file.display());
        return ExitCode::from(1);
    }
    match surety_syntax::normalize::canonical_file(&graph, idx) {
        Ok(c) => {
            println!("{}", c.text);
            println!("hash {}", c.hash);
            for p in &graph.file(idx).pins {
                println!("pinned {} {:?}: {}", p.line, p.kind, p.text);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}: {e}", file.display());
            ExitCode::from(1)
        }
    }
}

fn cmd_check(file: &Path, perf: Option<&Path>) -> ExitCode {
    let perf = perf
        .map(Path::to_path_buf)
        .unwrap_or_else(|| file.with_extension("py"));
    match surety::check_file(file, &perf) {
        Ok(report) => {
            print!("{}", surety::render(&report));
            if report.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}
