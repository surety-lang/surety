#!/usr/bin/env python3
"""Experiment 1: context audit over Claude Code transcripts.

Reads every session transcript under ~/.claude/projects (or the paths given),
and answers the spec's question: is tool output, and re-reading in particular,
the dominant share of what an agent spends its context on?

Two kinds of number, kept apart:

  chars   what entered the context, by source, measured in characters of
          transcript content (a stable proxy; ~4 chars per token). Sources:
          user text, assistant text, assistant thinking, tool calls (the
          arguments), tool results (by tool name).
  tokens  what the API billed, from each assistant turn's usage block:
          new input (input + cache_creation), cache reads, output. New input
          is the context that had not been seen before that turn; it is the
          number the token thesis is about.

Re-reads: a Read of a file path already Read earlier in the same session, and
the characters those repeat results cost. Grep/Glob results are counted as
tool output but not as re-reads.

Usage:
  python3 context_audit.py                      # all projects, last 21 days
  python3 context_audit.py --days 14
  python3 context_audit.py --project surety     # substring of the project dir
  python3 context_audit.py --by-project         # one line per project: the both-doors re-read share
  python3 context_audit.py --exclude foo        # leave out projects whose directory name contains foo
  python3 context_audit.py path/to/session.jsonl ...
  python3 context_audit.py --json > audit.json

Nothing is sent anywhere; the script only reads files and prints.
"""
import argparse, collections, datetime as dt, glob, json, os, sys

READ_TOOLS = {"Read"}
SEARCH_TOOLS = {"Grep", "Glob", "LS"}

import re
BASH_KINDS = [
    ("test",    re.compile(r"\b(pytest|cargo test|npm test|pnpm test|yarn test|jest|vitest|go test|make test|unittest|tox|nox)\b")),
    ("build",   re.compile(r"\b(cargo build|cargo check|cargo clippy|npm run build|pnpm build|tsc|make\b|docker build|go build|webpack|vite build)")),
    ("git",     re.compile(r"(^|[;&|]\s*)git\b")),
    ("package", re.compile(r"\b(pip install|pip3 install|npm install|npm i\b|pnpm add|pnpm install|cargo add|brew install|apt)\b")),
    ("write",   re.compile(r"(<<\s*['\"]?EOF|<<\s*['\"]?\w+['\"]?\n|\btee\b|\bcat\s*>)")),
    ("run",     re.compile(r"(^|[;&|]\s*)(python3?|node|cargo run|uv run|poetry run|\./[\w./-]+)\b")),
    ("inspect", re.compile(r"(^|[;&|]\s*)(ls|cat|head|tail|grep|rg|find|wc|sed|awk|tree|stat|du|echo)\b")),
]

PATH_RX = re.compile(r"(?<![\w@-])((?:~|\.{0,2}/)?[\w.\-]+(?:/[\w.\-]+)+|[\w\-]+\.(?:rs|py|ts|tsx|js|jsx|md|toml|json|yaml|yml|txt|sur|html|css|sh|go|java|c|h|cpp))(?![\w/])")
READ_CMDS = re.compile(r"(^|[;&|]\s*)(cat|head|tail|sed|less|more|bat|grep|rg|awk)\b")

def bash_read_paths(cmd):
    """Paths a shell command reads (cat/head/tail/sed/grep ...). Approximate: tokens that look like paths."""
    if not cmd or "<<" in cmd or not READ_CMDS.search(cmd):
        return []
    out = []
    for m in PATH_RX.finditer(cmd):
        tok = m.group(1)
        if tok.startswith("-"):
            continue
        out.append(tok)
    return out

def path_key(p):
    """Last two components of a path: tolerates relative vs absolute spellings, limits basename collisions."""
    parts = [x for x in p.replace("\\", "/").split("/") if x and x not in (".", "..", "~")]
    return "/".join(parts[-2:]) if parts else p

def bash_kind(cmd):
    for name, rx in BASH_KINDS:
        if rx.search(cmd or ""):
            return name
    return "other"


def content_blocks(msg):
    c = msg.get("content")
    if isinstance(c, str):
        return [{"type": "text", "text": c}]
    return [b for b in (c or []) if isinstance(b, dict)]


def block_chars(b):
    t = b.get("type")
    if t == "text":
        return len(b.get("text") or "")
    if t == "thinking":
        return len(b.get("thinking") or "")
    if t == "tool_use":
        try:
            return len(json.dumps(b.get("input"), ensure_ascii=False))
        except Exception:
            return 0
    if t == "tool_result":
        c = b.get("content")
        if isinstance(c, str):
            return len(c)
        n = 0
        for x in c or []:
            if isinstance(x, dict):
                n += len(x.get("text") or "") if x.get("type") == "text" else 0
            elif isinstance(x, str):
                n += len(x)
        return n
    return 0


def audit_session(path):
    s = {
        "path": path,
        "project": os.path.basename(os.path.dirname(path)),
        "turns": 0,
        "chars": collections.Counter(),       # by source
        "tool_result_chars": collections.Counter(),  # by tool name
        "tool_call_chars": collections.Counter(),    # by tool name (Write/Edit = code the agent wrote)
        "bash_result_chars": collections.Counter(),  # by command kind
        "bash_call_chars": collections.Counter(),
        "bash_calls": collections.Counter(),
        "tool_calls": collections.Counter(),
        "reads": 0, "rereads": 0, "reread_chars": 0, "files_read": 0,
        "bash_reads": 0, "bash_rereads": 0, "bash_reread_chars": 0,
        "tokens": collections.Counter(),      # new_input, cache_read, output
        "first": None, "last": None,
    }
    tool_name_by_id = {}
    seen_paths = collections.Counter()
    pending_read = {}  # tool_use_id -> path
    bash_kind_by_id = {}
    pending_bash_read = {}  # tool_use_id -> [paths]
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            try:
                r = json.loads(line)
            except Exception:
                continue
            ts = r.get("timestamp")
            if ts:
                s["first"] = s["first"] or ts
                s["last"] = ts
            t = r.get("type")
            m = r.get("message")
            if not isinstance(m, dict):
                continue
            if t == "assistant":
                s["turns"] += 1
                u = m.get("usage") or {}
                s["tokens"]["new_input"] += (u.get("input_tokens") or 0) + (u.get("cache_creation_input_tokens") or 0)
                s["tokens"]["cache_read"] += u.get("cache_read_input_tokens") or 0
                s["tokens"]["output"] += u.get("output_tokens") or 0
                for b in content_blocks(m):
                    k = b.get("type")
                    n = block_chars(b)
                    if k == "text":
                        s["chars"]["assistant_text"] += n
                    elif k == "thinking":
                        s["chars"]["assistant_thinking"] += n
                    elif k == "tool_use":
                        s["chars"]["tool_call"] += n
                        name = b.get("name") or "?"
                        tool_name_by_id[b.get("id")] = name
                        s["tool_calls"][name] += 1
                        s["tool_call_chars"][name] += n
                        if name in READ_TOOLS:
                            p = (b.get("input") or {}).get("file_path")
                            if p:
                                pending_read[b.get("id")] = p
                        if name == "Bash":
                            kind = bash_kind((b.get("input") or {}).get("command"))
                            bash_kind_by_id[b.get("id")] = kind
                            s["bash_calls"][kind] += 1
                            s["bash_call_chars"][kind] += n
                            paths = bash_read_paths((b.get("input") or {}).get("command"))
                            if paths:
                                pending_bash_read[b.get("id")] = paths
            elif t == "user":
                for b in content_blocks(m):
                    k = b.get("type")
                    n = block_chars(b)
                    if k == "text":
                        s["chars"]["user_text"] += n
                    elif k == "tool_result":
                        tid = b.get("tool_use_id")
                        name = tool_name_by_id.get(tid, "?")
                        s["chars"]["tool_result"] += n
                        s["tool_result_chars"][name] += n
                        if tid in bash_kind_by_id:
                            s["bash_result_chars"][bash_kind_by_id[tid]] += n
                        if tid in pending_bash_read:
                            paths = pending_bash_read.pop(tid)
                            s["bash_reads"] += 1
                            if any(seen_paths[path_key(p)] for p in paths):
                                s["bash_rereads"] += 1
                                s["bash_reread_chars"] += n
                            for p in paths:
                                seen_paths[path_key(p)] += 1
                        if tid in pending_read:
                            p = pending_read.pop(tid)
                            s["reads"] += 1
                            key = path_key(p)
                            if seen_paths[key]:
                                s["rereads"] += 1
                                s["reread_chars"] += n
                            seen_paths[key] += 1
    s["files_read"] = len(seen_paths)
    return s


def merge(sessions):
    tot = {
        "sessions": len(sessions), "turns": 0,
        "chars": collections.Counter(), "tool_result_chars": collections.Counter(),
        "tool_call_chars": collections.Counter(),
        "bash_result_chars": collections.Counter(), "bash_call_chars": collections.Counter(),
        "bash_calls": collections.Counter(),
        "tool_calls": collections.Counter(), "reads": 0, "rereads": 0,
        "reread_chars": 0, "tokens": collections.Counter(),
        "bash_reads": 0, "bash_rereads": 0, "bash_reread_chars": 0,
    }
    for s in sessions:
        tot["turns"] += s["turns"]
        tot["chars"].update(s["chars"]); tot["tool_result_chars"].update(s["tool_result_chars"])
        tot["tool_call_chars"].update(s["tool_call_chars"])
        tot["bash_result_chars"].update(s["bash_result_chars"]); tot["bash_call_chars"].update(s["bash_call_chars"])
        tot["bash_calls"].update(s["bash_calls"])
        tot["tool_calls"].update(s["tool_calls"]); tot["tokens"].update(s["tokens"])
        tot["reads"] += s["reads"]; tot["rereads"] += s["rereads"]; tot["reread_chars"] += s["reread_chars"]
        tot["bash_reads"] += s["bash_reads"]; tot["bash_rereads"] += s["bash_rereads"]; tot["bash_reread_chars"] += s["bash_reread_chars"]
    return tot


def pct(a, b):
    return 0.0 if not b else 100.0 * a / b


def report(name, tot):
    c = tot["chars"]; total = sum(c.values()) or 1
    tr = c["tool_result"]
    print(f"\n== {name}: {tot['sessions']} sessions, {tot['turns']} assistant turns ==")
    print("context by source (chars)            share")
    for k in ["tool_result", "tool_call", "assistant_text", "assistant_thinking", "user_text"]:
        print(f"  {k:<32}{c[k]:>12,}  {pct(c[k], total):5.1f}%")
    print("tool results by tool (chars)         share of all context")
    for k, v in tot["tool_result_chars"].most_common(8):
        print(f"  {k:<32}{v:>12,}  {pct(v, total):5.1f}%   calls {tot['tool_calls'][k]}")
    print("tool calls by tool (chars of arguments; Write/Edit is code the agent wrote)")
    for k, v in tot["tool_call_chars"].most_common(6):
        print(f"  {k:<32}{v:>12,}  {pct(v, total):5.1f}%   calls {tot['tool_calls'][k]}")
    print("Bash by command kind (result chars / argument chars, share of all context)")
    for k in ["test", "build", "run", "git", "package", "write", "inspect", "other"]:
        r, c = tot["bash_result_chars"][k], tot["bash_call_chars"][k]
        if r or c:
            print(f"  {k:<10}{r:>12,} {pct(r, total):5.1f}%   {c:>12,} {pct(c, total):5.1f}%   calls {tot['bash_calls'][k]}")
    print(f"re-reads: {tot['rereads']} of {tot['reads']} Reads ({pct(tot['rereads'], tot['reads']):.1f}%), "
          f"{tot['reread_chars']:,} chars = {pct(tot['reread_chars'], total):.1f}% of all context, "
          f"{pct(tot['reread_chars'], tr):.1f}% of tool output")
    br, brr, brc = tot["bash_reads"], tot["bash_rereads"], tot["bash_reread_chars"]
    print(f"reads via Bash (cat/head/tail/sed/grep on a path): {br}; of which re-reads of a path seen before: {brr} "
          f"({pct(brr, br):.1f}%), {brc:,} chars = {pct(brc, total):.1f}% of all context")
    allr, allrr, allrc = tot["reads"] + br, tot["rereads"] + brr, tot["reread_chars"] + brc
    print(f"re-reads, both doors: {allrr} of {allr} ({pct(allrr, allr):.1f}%), {allrc:,} chars = {pct(allrc, total):.1f}% of all context")
    k = tot["tokens"]
    print(f"tokens billed: new input {k['new_input']:,}  cache read {k['cache_read']:,}  output {k['output']:,}")
    verdict = "tool output IS the dominant share" if pct(tr, total) >= 50 else "tool output is NOT the dominant share"
    print(f"verdict on the kill criterion: {verdict} ({pct(tr, total):.1f}% of context chars)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="*")
    ap.add_argument("--days", type=int, default=21)
    ap.add_argument("--project", default=None, help="substring of the project directory name")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--by-project", action="store_true", help="one line per project, both doors, largest re-read share first")
    ap.add_argument("--exclude", action="append", default=[], metavar="SUBSTRING", help="leave out every project whose directory name contains this; repeatable")
    a = ap.parse_args()
    paths = a.paths or sorted(glob.glob(os.path.expanduser("~/.claude/projects/*/*.jsonl")))
    cutoff = (dt.datetime.now(dt.timezone.utc) - dt.timedelta(days=a.days)).timestamp()
    sessions = []
    for p in paths:
        if a.project and a.project not in os.path.basename(os.path.dirname(p)):
            continue
        if any(x in os.path.basename(os.path.dirname(p)) for x in a.exclude):
            continue
        if not a.paths and os.path.getmtime(p) < cutoff:
            continue
        try:
            s = audit_session(p)
        except Exception as e:
            print(f"skip {p}: {e}", file=sys.stderr); continue
        if s["turns"]:
            sessions.append(s)
    if a.json:
        print(json.dumps({"sessions": sessions, "total": merge(sessions)}, default=lambda o: dict(o) if isinstance(o, collections.Counter) else str(o), indent=1))
        return
    by_project = collections.defaultdict(list)
    for s in sessions:
        by_project[s["project"]].append(s)
    if a.by_project:
        # A worktree's sessions count under their project.
        grouped = collections.defaultdict(list)
        for proj, ss in by_project.items():
            name = proj.split("--claude-worktrees-")[0]
            for prefix in ("-Users-bennett-projects-", "-Users-bennett-"):
                if name.startswith(prefix):
                    name = name[len(prefix):]
                    break
            grouped[name].extend(ss)
        lines_out = []
        for proj, ss in grouped.items():
            tot = merge(ss)
            c = tot["chars"]; total = sum(c.values()) or 1
            allr = tot["reads"] + tot["bash_reads"]
            allrr = tot["rereads"] + tot["bash_rereads"]
            allrc = tot["reread_chars"] + tot["bash_reread_chars"]
            lines_out.append((pct(allrc, total), proj, tot["sessions"], tot["turns"], total,
                              pct(c["tool_result"], total), allrr, allr, pct(allrr, allr), allrc))
        lines_out.sort(reverse=True)
        print(f"{'project':<42}{'sessions':>9}{'turns':>7}{'context chars':>15}{'tool out':>10}{'re-reads, both doors':>24}{'chars':>12}{'share':>8}")
        for share, proj, n, turns, total, tool, rr, r, rrp, rc in lines_out:
            print(f"{proj[:41]:<42}{n:>9}{turns:>7}{total:>15,}{tool:>9.1f}%{rr:>12} of {r:<8}{rc:>12,}{share:>7.1f}%")
        return
    for proj, ss in sorted(by_project.items(), key=lambda kv: -len(kv[1])):
        report(proj, merge(ss))
    report("ALL PROJECTS", merge(sessions))


if __name__ == "__main__":
    main()
