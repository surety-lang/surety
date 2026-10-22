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
  surety pin <file.sur> [<perf.py>]     check and pin: the node, the performance and the record go
                                        into the graph; with no performance, the obligation alone
                                        (open). Siblings a property calls must be pinned first
  surety show <name | hash>             a node: its state, its prose, its presentations, its
                                        performances with their records, and one hop each way
  surety status                         every name in the namespace with its state; stale where
                                        the source file now hashes to something else
  surety verify                         the kernel over every record; --rerun checks each again and
                                        requires its evidence byte-identical
  surety render <file.sur>              the formatter: the file printed back from its syntax tree, prose
                                        wrapped at 100 columns, every comment where it was; --fresh-names
                                        renames every binder, which must not change the hash
  surety lsp                            the language server over stdio: diagnostics equal to parse's
                                        on every change, hover, definition, symbols; --graph for
                                        the state of a sibling obligation on hover
  surety diff <file.sur> <old.py> <new.py>
                                        the behavioural diff: both performances on the obligation's
                                        examples and generated inputs, and the inputs on which their
                                        outputs differ under the order claims, each classed against
                                        the obligation; graph references resolve beside each
                                        performance. Exit 0 when they agree everywhere, 1 when not
  surety --version | --help

Flags:
  --json          one JSON object per finding, then one summary object
  --strict        check: findings on generated inputs fail the check too
  --full          check: the bucket detail; mutants: the behaviour-first detail; diff: every
                  difference, not the first three
  --budget <n>    generated inputs per obligation (default 500; 0 is slice 1 behaviour)
  --candidates <n>  output candidates the search tries per input (default 600)
  --graph <dir>   the graph directory (default: graph)
  --rerun         verify: re-run every record's check
  --fresh-names   render: binders renamed in order of appearance";

#[derive(Default)]
struct Flags {
    json: bool,
    strict: bool,
    full: bool,
    budget: Option<usize>,
    candidates: Option<usize>,
    graph: Option<String>,
    rerun: bool,
    fresh_names: bool,
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
            "--rerun" => flags.rerun = true,
            "--fresh-names" => flags.fresh_names = true,
            "--graph" => {
                i += 1;
                match args.get(i) {
                    Some(d) => flags.graph = Some(d.clone()),
                    None => {
                        eprintln!("--graph needs a directory");
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
        ["pin", file] => cmd_pin(Path::new(file), None, &flags),
        ["pin", file, perf] => cmd_pin(Path::new(file), Some(Path::new(perf)), &flags),
        ["show", key] => cmd_show(key, &flags),
        ["status"] => cmd_status(&flags),
        ["verify"] => cmd_verify(&flags),
        ["render", file] => cmd_render(Path::new(file), &flags),
        ["lsp"] => match surety_lsp::run(flags.graph.as_ref().map(PathBuf::from)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("surety lsp: {e}");
                ExitCode::from(2)
            }
        },
        ["diff", file, old, new] => {
            cmd_diff(Path::new(file), Path::new(old), Path::new(new), &flags)
        }
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

fn cmd_diff(file: &Path, old: &Path, new: &Path, flags: &Flags) -> ExitCode {
    match surety::diff_files(file, old, new, flags.budget()) {
        Ok(report) => {
            if flags.json {
                print!("{}", surety::render_diff_json(&report));
            } else {
                print!("{}", surety::render_diff(&report, flags.full));
            }
            if report.same() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn store(flags: &Flags) -> surety_graph::Store {
    surety_graph::Store::open(Path::new(flags.graph.as_deref().unwrap_or("graph")))
}

fn cmd_pin(file: &Path, perf: Option<&Path>, flags: &Flags) -> ExitCode {
    let store = store(flags);
    match surety_graph::pin(
        &store,
        file,
        perf,
        flags.budget(),
        surety::TIMEOUT,
        surety::rendered,
    ) {
        Ok(pinned) => {
            if flags.json {
                if let Some(r) = &pinned.report {
                    print!("{}", surety::render_json(r));
                }
                println!("{}", surety::pin_json(&pinned));
            } else {
                if let Some(r) = &pinned.report {
                    print!("{}", surety::render_with(r, flags.full));
                }
                println!("{}", surety::pin_line(&pinned));
            }
            match pinned.state {
                surety_graph::PinState::Findings => ExitCode::from(1),
                _ => ExitCode::SUCCESS,
            }
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn cmd_show(key: &str, flags: &Flags) -> ExitCode {
    match surety_graph::show(&store(flags), key) {
        Ok(view) => {
            if flags.json {
                println!("{}", surety::show_json(&view));
            } else {
                print!("{}", surety::render_show(&view));
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn cmd_status(flags: &Flags) -> ExitCode {
    match surety_graph::status(&store(flags)) {
        Ok(rows) => {
            if flags.json {
                for j in surety::status_json(&rows) {
                    println!("{j}");
                }
            } else {
                print!("{}", surety::render_status(&rows));
            }
            let moved = rows.iter().any(|r| {
                matches!(
                    r.state,
                    surety_graph::State::Stale { .. } | surety_graph::State::Missing
                )
            });
            if moved {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn cmd_verify(flags: &Flags) -> ExitCode {
    match surety_graph::verify(&store(flags), flags.rerun, surety::rendered) {
        Ok(vs) => {
            if flags.json {
                for j in surety::verify_json(&vs) {
                    println!("{j}");
                }
            } else {
                print!("{}", surety::render_verify(&vs, flags.rerun));
            }
            if vs.iter().all(|v| v.result.is_ok()) {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn cmd_render(file: &Path, flags: &Flags) -> ExitCode {
    let text = match fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {}: {e}", file.display());
            return ExitCode::from(2);
        }
    };
    match surety_syntax::parse(&text) {
        Ok(f) => {
            print!("{}", surety_syntax::render::file(&f, flags.fresh_names));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}: {e}", file.display());
            ExitCode::from(2)
        }
    }
}
