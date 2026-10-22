//! Runs a Python performance in a subprocess over a JSON boundary.
//!
//! One process per module, kept alive across calls. Each request is one
//! line of JSON on stdin; each response one line on stdout. No sandboxing
//! beyond the subprocess and a per-call timeout. A process says it is ready
//! once the module has imported, so a module that is missing or fails to
//! import is known when it is started, not on its first call.

use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::thread;
use std::time::Duration;

use serde_json::Value;

/// The line a process writes once its module has imported.
const READY: &str = r#"{"ready": true}"#;

const DRIVER: &str = r#"
import importlib.util, json, sys, traceback
READY = '{"ready": true}'
from datetime import datetime, timezone
path = sys.argv[1]
spec = importlib.util.spec_from_file_location("performance", path)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
sys.stdout.write(READY + "\n")
sys.stdout.flush()


def thaw(v):
    # A Time crosses tagged, {"$time": "<RFC 3339 UTC>"}: deliver it as an
    # aware UTC datetime. Everything else is plain JSON.
    if isinstance(v, dict):
        if len(v) == 1 and "$time" in v:
            s = v["$time"]
            if s.endswith("Z"):
                s = s[:-1] + "+00:00"
            t = datetime.fromisoformat(s)
            if t.tzinfo is None:
                t = t.replace(tzinfo=timezone.utc)
            return t.astimezone(timezone.utc)
        return {k: thaw(x) for k, x in v.items()}
    if isinstance(v, list):
        return [thaw(x) for x in v]
    return v


def freeze(v):
    # A datetime returns as RFC 3339 UTC text; a naive one is read as UTC.
    if isinstance(v, datetime):
        if v.tzinfo is None:
            v = v.replace(tzinfo=timezone.utc)
        v = v.astimezone(timezone.utc)
        s = "%04d-%02d-%02dT%02d:%02d:%02d" % (v.year, v.month, v.day, v.hour, v.minute, v.second)
        if v.microsecond:
            s += (".%06d" % v.microsecond).rstrip("0")
        return s + "Z"
    raise TypeError("Object of type %s is not JSON serializable" % type(v).__name__)


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
            resp = {"ok": fn(*thaw(req["args"]))}
        out = json.dumps(resp, default=freeze)
    except BaseException:
        out = json.dumps({"error": traceback.format_exc()})
    sys.stdout.write(out + "\n")
    sys.stdout.flush()
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// Python could not be started.
    Spawn(String),
    /// There is no module at the path.
    Missing(PathBuf),
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
            RunError::Missing(p) => write!(f, "no performance at {}", p.display()),
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

/// Python processes started in this process, for the tests that hold a
/// run to one start per module.
static STARTS: AtomicUsize = AtomicUsize::new(0);

pub fn starts() -> usize {
    STARTS.load(Ordering::Relaxed)
}

impl Runner {
    /// Start a process for the module at `path` and wait until the module
    /// has imported: a missing module, one that fails to import, or one
    /// whose import outlasts the timeout is an error here.
    pub fn new(path: &Path, timeout: Duration) -> Result<Runner, RunError> {
        if !path.is_file() {
            return Err(RunError::Missing(path.to_path_buf()));
        }
        let python = std::env::var("SURETY_PYTHON").unwrap_or_else(|_| "python3".to_string());
        STARTS.fetch_add(1, Ordering::Relaxed);
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
        let mut runner = Runner {
            child,
            stdin,
            lines,
            stderr,
            timeout,
            path: path.to_path_buf(),
            dead: false,
        };
        runner.ready()?;
        Ok(runner)
    }

    /// Wait for the ready line; anything the module printed while importing
    /// comes first and is passed over.
    fn ready(&mut self) -> Result<(), RunError> {
        loop {
            match self.lines.recv_timeout(self.timeout) {
                Ok(Some(l)) if l == READY => return Ok(()),
                Ok(Some(_)) => {}
                Ok(None) | Err(RecvTimeoutError::Disconnected) => {
                    self.dead = true;
                    let _ = self.child.wait();
                    return Err(RunError::Exited(self.collect_stderr(String::new())));
                }
                Err(RecvTimeoutError::Timeout) => {
                    self.dead = true;
                    return Err(RunError::Timeout(self.timeout));
                }
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether the process has gone (a timeout kills it; a raise does not).
    pub fn is_dead(&self) -> bool {
        self.dead
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

const INSPECT: &str = r#"
import ast, hashlib, json, sys
path = sys.argv[1]
src = open(path, encoding="utf-8").read()
tree = ast.parse(src)
imports = []
for node in ast.walk(tree):
    if isinstance(node, ast.Import):
        for a in node.names:
            imports.append(a.name.split(".")[0])
    elif isinstance(node, ast.ImportFrom):
        imports.append((node.module or "").split(".")[0] or ".")
imports = list(dict.fromkeys(imports))
stdlib = set(getattr(sys, "stdlib_module_names", ()))
deps = [m for m in imports if m not in stdlib]
functions = sum(isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) for n in ast.walk(tree))
lines = len(src.splitlines())

def strip_doc(body):
    if (body and isinstance(body[0], ast.Expr)
            and isinstance(getattr(body[0], "value", None), ast.Constant)
            and isinstance(body[0].value.value, str)):
        body = body[1:]
    return body or [ast.Pass()]

class Collector(ast.NodeVisitor):
    # Pass one: every local binding, in order of first appearance. Locals,
    # comprehension variables, lambda and nested-function parameters and
    # exception names are bindings; the function's own parameters and every
    # global name are kept, since they are the signature.
    def __init__(self, keep):
        self.keep = keep
        self.map = {}
    def fresh(self, name):
        if name not in self.keep and name not in self.map:
            self.map[name] = "v%d" % len(self.map)
    def visit_arg(self, node):
        self.fresh(node.arg)
    def visit_Name(self, node):
        if isinstance(node.ctx, ast.Store):
            self.fresh(node.id)
    def visit_ExceptHandler(self, node):
        if node.name:
            self.fresh(node.name)
        self.generic_visit(node)

class Renamer(ast.NodeTransformer):
    # Pass two: rename every occurrence, strip docstrings and annotations.
    def __init__(self, mapping):
        self.map = mapping
    def visit_arg(self, node):
        node.arg = self.map.get(node.arg, node.arg)
        node.annotation = None
        return node
    def visit_Name(self, node):
        node.id = self.map.get(node.id, node.id)
        return node
    def visit_ExceptHandler(self, node):
        if node.name:
            node.name = self.map.get(node.name, node.name)
        self.generic_visit(node)
        return node
    def visit_FunctionDef(self, node):
        node.body = strip_doc(node.body)
        node.returns = None
        self.generic_visit(node)
        return node
    visit_AsyncFunctionDef = visit_FunctionDef

tree.body = strip_doc(tree.body)
for node in tree.body:
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
        a = node.args
        params = {x.arg for x in a.posonlyargs + a.args + a.kwonlyargs}
        if a.vararg:
            params.add(a.vararg.arg)
        if a.kwarg:
            params.add(a.kwarg.arg)
        node.body = strip_doc(node.body)
        node.returns = None
        collector = Collector(params)
        collector.generic_visit(node)
        Renamer(collector.map).generic_visit(node)
def dump(node):
    # Our own dump, the same on every Python: ast.dump's shape changed in
    # 3.13 (empty fields are left out), and a hash is an identity only if
    # every interpreter computes the same one.
    if isinstance(node, ast.AST):
        parts = []
        for f in node._fields:
            v = getattr(node, f, None)
            if v is None or (isinstance(v, list) and not v):
                continue
            parts.append(f + "=" + dump(v))
        return type(node).__name__ + "(" + ", ".join(parts) + ")"
    if isinstance(node, list):
        return "[" + ", ".join(dump(x) for x in node) + "]"
    return repr(node)

dumped = dump(tree)
print(json.dumps({
    "hash": hashlib.sha256(dumped.encode("utf-8")).hexdigest(),
    "imports": imports,
    "dependencies": deps,
    "lines": lines,
    "functions": functions,
}))
"#;

/// What the Python target says about a performance's source: the
/// structural hash (formatting and comments stripped, locals renamed in
/// order of appearance), the modules it imports, which of those are not in
/// the standard library, and its size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    pub hash: String,
    pub imports: Vec<String>,
    pub dependencies: Vec<String>,
    pub lines: usize,
    pub functions: usize,
}

/// Inspect a performance module without running it.
/// The interpreter's version, for a record's provenance.
pub fn python_version() -> Result<String, RunError> {
    let python = std::env::var("SURETY_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let out = Command::new(&python)
        .arg("-I")
        .arg("-c")
        .arg("import sys; print('%d.%d.%d' % sys.version_info[:3])")
        .output()
        .map_err(|e| RunError::Spawn(format!("{python}: {e}")))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn inspect(path: &Path) -> Result<Inspection, RunError> {
    let python = std::env::var("SURETY_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let out = Command::new(&python)
        .arg("-I")
        .arg("-c")
        .arg(INSPECT)
        .arg(path)
        .output()
        .map_err(|e| RunError::Spawn(format!("{python}: {e}")))?;
    if !out.status.success() {
        return Err(RunError::Exited(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let v: Value = serde_json::from_str(text.trim())
        .map_err(|e| RunError::Protocol(format!("{e}: {text}")))?;
    let strings = |key: &str| -> Vec<String> {
        v[key]
            .as_array()
            .map(|xs| {
                xs.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    Ok(Inspection {
        hash: v["hash"].as_str().unwrap_or_default().to_string(),
        imports: strings("imports"),
        dependencies: strings("dependencies"),
        lines: v["lines"].as_u64().unwrap_or(0) as usize,
        functions: v["functions"].as_u64().unwrap_or(0) as usize,
    })
}

impl Drop for Runner {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
