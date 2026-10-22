//! Loading a directory of `.sur` files as one graph.

use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::File;
use crate::error::{Error, ErrorKind};
use crate::parser::parse;
use crate::resolve::Graph;

/// A file's stem: how the graph names it and how its performance is found.
pub fn stem(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Load the directory holding `file` as a graph and return the index of
/// `file` in it (always 0). If `file` does not parse, the error is returned
/// alone; siblings that do not parse are skipped, since they are not this
/// file's problem.
pub fn load_with_siblings(file: &Path) -> Result<(Graph, usize), Vec<(PathBuf, Error)>> {
    load_with_siblings_text(file, &read_source(file)?)
}

/// A file's text, or the error `load_with_siblings` reports when it cannot
/// be read.
pub fn read_source(file: &Path) -> Result<String, Vec<(PathBuf, Error)>> {
    fs::read_to_string(file).map_err(|e| {
        vec![(
            file.to_path_buf(),
            Error::new(ErrorKind::Syntax, 0, e.to_string()),
        )]
    })
}

/// The same, with the file's text given rather than read: what an editor
/// holds unsaved, parsed against the siblings on disk.
pub fn load_with_siblings_text(
    file: &Path,
    text: &str,
) -> Result<(Graph, usize), Vec<(PathBuf, Error)>> {
    load_with_siblings_from(file, text, |p| parse(&fs::read_to_string(p).ok()?).ok())
}

/// The same, with each sibling's parse from `sibling`, None to skip it: a
/// server keeps the parses between requests.
pub fn load_with_siblings_from(
    file: &Path,
    text: &str,
    mut sibling: impl FnMut(&Path) -> Option<File>,
) -> Result<(Graph, usize), Vec<(PathBuf, Error)>> {
    let dir = file.parent().unwrap_or(Path::new("."));
    let own = parse(text).map_err(|e| vec![(file.to_path_buf(), e)])?;
    let own_stem = stem(file);
    let mut files = vec![(own_stem.clone(), own)];
    if let Ok(rd) = fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "sur") && stem(p) != own_stem)
            .collect();
        paths.sort();
        for p in paths {
            if let Some(f) = sibling(&p) {
                files.push((stem(&p), f));
            }
        }
    }
    Ok((Graph::new(files), 0))
}
