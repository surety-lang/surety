use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use surety_check::Budget;
use surety_syntax::extract::extract;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = "\
Usage:
  surety extract <markdown> <out-dir>   cut the obligations out of the authored markdown
  surety parse <file.sur>...            parse and check each file (siblings form the graph)
  surety hash <file.sur>                print the canonical form and its hash
  surety check <file.sur> [<perf.py>]   run the examples, the properties over the examples and
                                        over generated inputs, determinism and totality against
                                        a performance (default: <file>.py next to the obligation),
                                        and print the evidence record
  surety mutants <file.sur>...          the underdetermination check: the mutant library against
                                        the obligation alone, over the same inputs, and the search
                                        for inputs with two passing outputs; a performance next to
                                        the file is used only where a property calls the obligation
  surety --version | --help

Flags:
  --json          one JSON object per finding, then one summary object
  --strict        check: findings on generated inputs fail the check too
  --full          check: the bucket detail; mutants: the behaviour-first detail
  --budget <n>    generated inputs per obligation (default 500; 0 is slice 1 behaviour)
  --candidates <n>  output candidates the search tries per input (default 600)";

#[derive(Default)]
struct Flags {
    json: bool,
    strict: bool,
    full: bool,
    budget: Option<usize>,
    candidates: Option<usize>,
}

impl Flags {
    fn budget(&self) -> Budget {
        let mut b = Budget::default();
        if let Some(n) = self.budget {
            b.generated = n;
        }
        if let Some(n) = self.candidates {
            b.search_candidates = n;
        }
        b
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut flags = Flags::default();
    let mut positional: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--json" => flags.json = true,
            "--strict" => flags.strict = true,
            "--full" => flags.full = true,
            "--budget" => {
                i += 1;
                match args.get(i).and_then(|n| n.parse().ok()) {
                    Some(n) => flags.budget = Some(n),
                    None => {
                        eprintln!("--budget needs a number");
                        return ExitCode::from(2);
                    }
                }
            }
            "--candidates" => {
                i += 1;
                match args.get(i).and_then(|n| n.parse().ok()) {
                    Some(n) => flags.candidates = Some(n),
                    None => {
                        eprintln!("--candidates needs a number");
                        return ExitCode::from(2);
                    }
                }
            }
            "--version" => {
                println!("surety {VERSION}");
                return ExitCode::SUCCESS;
            }
            "--help" => {
                println!("{HELP}");
                return ExitCode::SUCCESS;
            }
            other if other.starts_with("--") => {
                eprintln!("unknown flag {other}\n{HELP}");
                return ExitCode::from(2);
            }
            other => positional.push(other),
        }
        i += 1;
    }
    match positional.as_slice() {
        [] => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        ["extract", md, out] => cmd_extract(Path::new(md), Path::new(out)),
        ["parse", files @ ..] if !files.is_empty() => cmd_parse(files),
        ["hash", file] => cmd_hash(Path::new(file)),
        ["check", file] => cmd_check(Path::new(file), None, &flags),
        ["check", file, perf] => cmd_check(Path::new(file), Some(Path::new(perf)), &flags),
        ["mutants", files @ ..] if !files.is_empty() => cmd_mutants(files, &flags),
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
                            "{}: ok {} props={} accepts={} examples={} abbrevs={} pins={}",
                            path.display(),
                            f.name,
                            f.properties.len(),
                            f.accepts.len(),
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

fn cmd_check(file: &Path, perf: Option<&Path>, flags: &Flags) -> ExitCode {
    let perf = perf
        .map(Path::to_path_buf)
        .unwrap_or_else(|| file.with_extension("py"));
    match surety::check_file_with(file, &perf, flags.budget()) {
        Ok(report) => {
            if flags.json {
                print!("{}", surety::render_json(&report));
            } else {
                print!("{}", surety::render_with(&report, flags.full));
            }
            let ok = if flags.strict {
                report.strict_passed()
            } else {
                report.passed()
            };
            if ok {
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

fn cmd_mutants(files: &[&str], flags: &Flags) -> ExitCode {
    let mut failed = false;
    let mut underdetermined = 0;
    for f in files {
        let file = Path::new(f);
        match surety::mutants_file(file, None, flags.budget()) {
            Ok(report) => {
                if flags.json {
                    print!("{}", surety::render_mutants_json(&report));
                } else {
                    print!("{}", surety::render_mutants(&report, flags.full));
                }
                if report.underdetermined() {
                    underdetermined += 1;
                }
            }
            Err(e) => {
                eprintln!("{}: {e}", file.display());
                failed = true;
            }
        }
    }
    if files.len() > 1 && !flags.json {
        println!(
            "underdetermination rate: {underdetermined} of {} obligations",
            files.len()
        );
    }
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
