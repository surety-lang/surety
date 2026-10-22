//! The siblings' parses, kept between requests. Every request loads the
//! file with its directory as one graph; a sibling's parse is kept while
//! its modification time and length stay as they were, so a keystroke
//! parses the text the editor holds and only the siblings that changed on
//! disk.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use surety_syntax::load::{load_with_siblings_from, read_source};
use surety_syntax::{Error, File, Graph};

/// A file read this soon after it was written could be written again in
/// the same tick of a coarse file system clock, at the same length: until
/// it was read this long after its modification time, its text is
/// compared as well.
const SETTLE: Duration = Duration::from_secs(2);

struct Kept {
    modified: SystemTime,
    len: u64,
    read_at: SystemTime,
    text: String,
    file: Option<File>,
}

#[derive(Default)]
pub struct Siblings {
    kept: HashMap<PathBuf, Kept>,
    parses: usize,
}

impl Siblings {
    /// A sibling's parse, None when it cannot be read or does not parse.
    fn sibling(&mut self, p: &Path) -> Option<File> {
        let stamp = fs::metadata(p)
            .ok()
            .and_then(|m| Some((m.modified().ok()?, m.len())));
        if let (Some(k), Some((modified, len))) = (self.kept.get(p), stamp) {
            let settled = k
                .read_at
                .duration_since(k.modified)
                .is_ok_and(|d| d >= SETTLE);
            if settled && k.modified == modified && k.len == len {
                return k.file.clone();
            }
        }
        let read_at = SystemTime::now();
        let Ok(text) = fs::read_to_string(p) else {
            self.kept.remove(p);
            return None;
        };
        let file = match self.kept.remove(p) {
            Some(k) if k.text == text => k.file,
            _ => {
                self.parses += 1;
                surety_syntax::parse(&text).ok()
            }
        };
        if let Some((modified, len)) = stamp {
            self.kept.insert(
                p.to_path_buf(),
                Kept {
                    modified,
                    len,
                    read_at,
                    text,
                    file: file.clone(),
                },
            );
        }
        file
    }

    /// `load_with_siblings_text`, the siblings' parses kept. A file gone
    /// from the directory is forgotten.
    pub fn load(
        &mut self,
        file: &Path,
        text: &str,
    ) -> Result<(Graph, usize), Vec<(PathBuf, Error)>> {
        let mut listed = HashSet::new();
        let loaded = load_with_siblings_from(file, text, |p| {
            listed.insert(p.to_path_buf());
            self.sibling(p)
        });
        if loaded.is_ok() {
            let dir = file.parent().unwrap_or(Path::new("."));
            self.kept
                .retain(|p, _| p.parent() != Some(dir) || listed.contains(p) || p.exists());
        }
        loaded
    }

    /// `load_with_siblings`: the file's own text as saved.
    pub fn load_saved(&mut self, file: &Path) -> Result<(Graph, usize), Vec<(PathBuf, Error)>> {
        let text = read_source(file)?;
        self.load(file, &text)
    }

    /// The siblings parsed from disk so far: what the test reads.
    pub fn parses(&self) -> usize {
        self.parses
    }
}
