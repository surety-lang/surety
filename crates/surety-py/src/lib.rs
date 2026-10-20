//! Runs a Python performance in a subprocess over a JSON boundary.
//!
//! One process per module, kept alive across calls. Each request is one
//! line of JSON on stdin; each response one line on stdout. No sandboxing
//! beyond the subprocess and a per-call timeout.

use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::thread;
use std::time::Duration;

use serde_json::Value;

const DRIVER: &str = r#"
import importlib.util, json, sys, traceback
path = sys.argv[1]
spec = importlib.util.spec_from_file_location("performance", path)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        req = json.loads(line)
        if "has" in req:
            resp = {"ok": callable(getattr(mod, req["has"], None))}
        else:
            fn = getattr(mod, req["fn"])
            resp = {"ok": fn(*req["args"])}
        out = json.dumps(resp)
    except BaseException:
        out = json.dumps({"error": traceback.format_exc()})
    sys.stdout.write(out + "\n")
    sys.stdout.flush()
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// Python could not be started.
    Spawn(String),
    /// The call did not answer within the timeout; the process was killed.
    Timeout(Duration),
    /// The process ended; typically the module failed to import.
    Exited(String),
    /// The performance raised; the payload is the traceback.
    Raised(String),
    /// The reply was not understood.
    Protocol(String),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Spawn(s) => write!(f, "cannot start python: {s}"),
            RunError::Timeout(d) => write!(f, "performance did not answer within {d:?}"),
            RunError::Exited(s) => write!(f, "performance process exited: {}", s.trim()),
            RunError::Raised(s) => write!(f, "performance raised:\n{}", s.trim_end()),
            RunError::Protocol(s) => write!(f, "bad reply from performance: {s}"),
        }
    }
}

impl std::error::Error for RunError {}

pub struct Runner {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<Option<String>>,
    stderr: Receiver<String>,
    timeout: Duration,
    path: PathBuf,
    dead: bool,
}

impl Runner {
    /// Start a process for the module at `path`.
    pub fn new(path: &Path, timeout: Duration) -> Result<Runner, RunError> {
        let python = std::env::var("SURETY_PYTHON").unwrap_or_else(|_| "python3".to_string());
        let mut child = Command::new(&python)
            .arg("-I")
            .arg("-c")
            .arg(DRIVER)
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RunError::Spawn(format!("{python}: {e}")))?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr_pipe = child.stderr.take().unwrap();
        let (tx, lines) = channel();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        if tx.send(Some(l)).is_err() {
                            return;
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = tx.send(None);
        });
        let (etx, stderr) = channel();
        thread::spawn(move || {
            let reader = BufReader::new(stderr_pipe);
            let mut all = String::new();
            for line in reader.lines().map_while(Result::ok) {
                all.push_str(&line);
                all.push('\n');
            }
            let _ = etx.send(all);
        });
        Ok(Runner {
            child,
            stdin,
            lines,
            stderr,
            timeout,
            path: path.to_path_buf(),
            dead: false,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn request(&mut self, req: Value) -> Result<Value, RunError> {
        if self.dead {
            return Err(RunError::Exited("process already gone".into()));
        }
        let mut line = req.to_string();
        line.push('\n');
        if let Err(e) = self
            .stdin
            .write_all(line.as_bytes())
            .and_then(|_| self.stdin.flush())
        {
            self.dead = true;
            return Err(RunError::Exited(self.collect_stderr(e.to_string())));
        }
        let reply = match self.lines.recv_timeout(self.timeout) {
            Ok(Some(l)) => l,
            Ok(None) | Err(RecvTimeoutError::Disconnected) => {
                self.dead = true;
                let _ = self.child.wait();
                return Err(RunError::Exited(self.collect_stderr(String::new())));
            }
            Err(RecvTimeoutError::Timeout) => {
                self.dead = true;
                let _ = self.child.kill();
                let _ = self.child.wait();
                return Err(RunError::Timeout(self.timeout));
            }
        };
        let v: Value = serde_json::from_str(&reply)
            .map_err(|e| RunError::Protocol(format!("{e}: {reply}")))?;
        let Value::Object(mut map) = v else {
            return Err(RunError::Protocol(reply));
        };
        if let Some(err) = map.remove("error") {
            return Err(RunError::Raised(err.as_str().unwrap_or("").to_string()));
        }
        map.remove("ok").ok_or(RunError::Protocol(reply))
    }

    fn collect_stderr(&self, prefix: String) -> String {
        let tail = self
            .stderr
            .recv_timeout(Duration::from_millis(500))
            .unwrap_or_default();
        if prefix.is_empty() {
            tail
        } else {
            format!("{prefix}\n{tail}")
        }
    }

    /// Whether the module defines a callable of that name.
    pub fn has(&mut self, name: &str) -> Result<bool, RunError> {
        let v = self.request(serde_json::json!({ "has": name }))?;
        Ok(v.as_bool().unwrap_or(false))
    }

    /// Call a function with positional JSON arguments.
    pub fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, RunError> {
        self.request(serde_json::json!({ "fn": name, "args": args }))
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
