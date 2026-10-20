//! The `surety` command: loads a corpus directory as one graph, parses,
//! hashes, and checks obligations against performances.

use std::fs;
use std::path::{Path, PathBuf};

use std::time::Duration;

use surety_check::{Checker, Report, Verdict};
use surety_syntax::{Error, Graph, parse};

/// Per-call timeout for a performance.
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// Every `.sur` file in a directory, parsed, as one graph. Parse errors are
/// returned per file so one bad file does not hide the others.
pub fn load_dir(dir: &Path) -> Result<Graph, Vec<(PathBuf, Error)>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| {
            vec![(
                dir.to_path_buf(),
                Error::new(surety_syntax::ErrorKind::Syntax, 0, e.to_string()),
            )]
        })?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "sur"))
        .collect();
    paths.sort();
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for p in paths {
        let text = match fs::read_to_string(&p) {
            Ok(t) => t,
            Err(e) => {
                errors.push((
                    p.clone(),
                    Error::new(surety_syntax::ErrorKind::Syntax, 0, e.to_string()),
                ));
                continue;
            }
        };
        match parse(&text) {
            Ok(f) => files.push((stem(&p), f)),
            Err(e) => errors.push((p.clone(), e)),
        }
    }
    if errors.is_empty() {
        Ok(Graph::new(files))
    } else {
        Err(errors)
    }
}

pub fn stem(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Load the directory holding `file` as a graph and return the index of
/// `file` in it. If `file` does not parse, the error is returned alone.
pub fn load_with_siblings(file: &Path) -> Result<(Graph, usize), Vec<(PathBuf, Error)>> {
    let dir = file.parent().unwrap_or(Path::new("."));
    let text = fs::read_to_string(file).map_err(|e| {
        vec![(
            file.to_path_buf(),
            Error::new(surety_syntax::ErrorKind::Syntax, 0, e.to_string()),
        )]
    })?;
    let own = parse(&text).map_err(|e| vec![(file.to_path_buf(), e)])?;
    let own_stem = stem(file);
    let mut files = vec![(own_stem.clone(), own)];
    if let Ok(rd) = fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "sur") && stem(p) != own_stem)
            .collect();
        paths.sort();
        for p in paths {
            // Siblings that do not parse are skipped: they are not this file's problem.
            if let Ok(t) = fs::read_to_string(&p) {
                if let Ok(f) = parse(&t) {
                    files.push((stem(&p), f));
                }
            }
        }
    }
    Ok((Graph::new(files), 0))
}

/// Check one obligation file against a performance module: examples and
/// properties.
pub fn check_file(file: &Path, performance: &Path) -> Result<Report, String> {
    let (graph, idx) = load_with_siblings(file).map_err(|errs| {
        errs.iter()
            .map(|(p, e)| format!("{}: {e}", p.display()))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    graph
        .check_file(idx)
        .map_err(|e| format!("{}: {e}", file.display()))?;
    let mut checker = Checker::new(&graph, idx, performance, TIMEOUT)?;
    Ok(checker.check())
}

/// A plain-text rendering of a report.
pub fn render(report: &Report) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}\n", report.obligation));
    for e in &report.examples {
        let label = match &e.note {
            Some(n) => format!("example {} (line {}, {n:?})", e.index + 1, e.line),
            None => format!("example {} (line {})", e.index + 1, e.line),
        };
        match &e.verdict {
            Verdict::Pass => s.push_str(&format!("  pass  {label}\n")),
            Verdict::Boundary(m) => s.push_str(&format!("  BOUNDARY  {label}: {m}\n")),
            Verdict::Mismatch(m) => s.push_str(&format!("  FAIL  {label}: {m}\n")),
            Verdict::Error(m) => s.push_str(&format!("  ERROR  {label}: {m}\n")),
        }
    }
    s
}
