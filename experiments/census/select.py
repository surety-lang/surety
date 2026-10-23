#!/usr/bin/env python3
"""Pick the census functions and write one problem file per function.

Writes experiments/census/functions.jsonl (one record per selected function:
id, source, module, name, qualname, version, signature in words, docstring
length) and experiments/census/skipped.jsonl (every public function in the
scanned modules that was left out, with the reason), and
experiments/census/problems/<id>.md (the docstring verbatim plus the
signature in words; no code body).

Selection rules (the brief's): pure (no IO, no mutation of arguments, no
randomness, no clock) and a docstring good enough to be a problem statement.
Classes are not functions and are skipped. Aliases and lazy twins of an
eager function are skipped as duplicates so a function is counted once.
"""
from __future__ import annotations

import importlib
import importlib.metadata as md
import inspect
import json
import pathlib
import re

HERE = pathlib.Path(__file__).resolve().parent
PROBLEMS = HERE / "problems"

# module -> (source label, dist name)
MODULES = {
    "more_itertools.more": ("more-itertools", "more-itertools"),
    "more_itertools.recipes": ("itertools recipes (more_itertools.recipes)", "more-itertools"),
    "toolz.itertoolz": ("toolz", "toolz"),
    "toolz.functoolz": ("toolz", "toolz"),
    "toolz.dicttoolz": ("toolz", "toolz"),
    "toolz.recipes": ("toolz", "toolz"),
    "boltons.iterutils": ("boltons", "boltons"),
    "boltons.strutils": ("boltons", "boltons"),
    "boltons.dictutils": ("boltons", "boltons"),
    "boltons.mathutils": ("boltons", "boltons"),
    "boltons.urlutils": ("boltons", "boltons"),
    "boltons.timeutils": ("boltons", "boltons"),
    "boltons.formatutils": ("boltons", "boltons"),
    "boltons.setutils": ("boltons", "boltons"),
    "boltons.funcutils": ("boltons", "boltons"),
    "django.utils.text": ("django", "django"),
    "django.utils.html": ("django", "django"),
    "django.utils.http": ("django", "django"),
    "django.utils.dateparse": ("django", "django"),
    "requests.utils": ("requests", "requests"),
    "humanize.number": ("humanize", "humanize"),
    "humanize.filesize": ("humanize", "humanize"),
    "humanize.lists": ("humanize", "humanize"),
    "humanize.time": ("humanize", "humanize"),
}

# name -> reason, per module. Anything public and not listed here is selected.
SKIP = {
    "more_itertools.more": {
        "consumer": "decorator; side effects (advances a coroutine)",
        "with_iter": "IO (context manager, file handles)",
        "raise_": "no docstring; raises",
        "side_effect": "side effects by definition",
        "interleave_randomly": "randomness",
        "sample": "randomness",
        "make_decorator": "returns a decorator; about functions, not data",
        "synchronized": "threading; side effects",
        "concurrent_tee": "threading; side effects",
        "random_ordered_range": "randomness",
        "iter_suppress": "input is an iterator that raises, not data",
    },
    "more_itertools.recipes": {
        "consume": "side effects (advances an iterator); IO in the docstring",
        "iter_except": "calls a function until it raises; effectful",
        "pairwise": "docstring is 'Wrapper for itertools.pairwise' (thin)",
        "padnone": "alias of pad_none",
        "random_product": "randomness",
        "random_permutation": "randomness",
        "random_combination": "randomness",
        "random_combination_with_replacement": "randomness",
        "random_derangement": "randomness",
        "loops": "thin docstring (an n-element iterable of unspecified values)",
    },
    "toolz.itertoolz": {
        "random_sample": "randomness",
        "getter": "no docstring",
        "isiterable": "type introspection, not a data relation",
    },
    "toolz.functoolz": {
        "instanceproperty": "class-like decorator",
        "do": "side effects by definition",
        "return_none": "thin docstring",
        "num_required_args": "introspection of code objects",
        "has_varargs": "introspection of code objects",
        "has_keywords": "introspection of code objects",
        "is_valid_args": "introspection of code objects",
        "is_partial_args": "introspection of code objects",
        "is_arity": "introspection of code objects",
    },
    "toolz.dicttoolz": {},
    "toolz.recipes": {},
    "boltons.iterutils": {
        "is_iterable": "type introspection, not a data relation",
        "is_scalar": "type introspection, not a data relation",
        "is_collection": "type introspection, not a data relation",
        "split_iter": "lazy twin of split",
        "lstrip_iter": "lazy twin of lstrip",
        "rstrip_iter": "lazy twin of rstrip",
        "strip_iter": "lazy twin of strip",
        "chunked_iter": "lazy twin of chunked",
        "pairwise_iter": "lazy twin of pairwise",
        "windowed_iter": "lazy twin of windowed",
        "unique_iter": "lazy twin of unique",
        "flatten_iter": "lazy twin of flatten",
        "backoff_iter": "randomness (jitter)",
        "default_visit": "no docstring",
        "default_enter": "no docstring",
        "default_exit": "no docstring",
    },
    "boltons.strutils": {
        "html2text": "depends on an HTML parser's behaviour",
        "gunzip_bytes": "compression; the docstring leans on zlib",
        "gzip_bytes": "compression; gzip output carries a timestamp",
    },
    "boltons.dictutils": {},
    "boltons.mathutils": {},
    "boltons.urlutils": {
        "to_unicode": "no docstring",
        "quote_path_part": "thin docstring",
        "quote_query_part": "thin docstring",
        "unquote_to_bytes": "thin docstring",
        "register_scheme": "mutates a global registry",
        "parse_qsl": "thin docstring",
    },
    "boltons.timeutils": {
        "parse_td": "alias of parse_timedelta",
        "decimal_relative_time": "clock (other defaults to now)",
        "relative_time": "clock (other defaults to now)",
    },
    "boltons.formatutils": {},
    "boltons.setutils": {"complement": "returns a wrapper object, not data"},
    "boltons.funcutils": {"*": "decorators and function introspection, not data relations"},
    "django.utils.text": {
        "add_truncation_text": "no docstring",
        "calculate_truncate_chars_length": "no docstring",
        "compress_string": "no docstring; gzip",
        "compress_sequence": "no docstring; gzip",
        "acompress_sequence": "no docstring; async",
        "format_lazy": "lazy evaluation wrapper",
    },
    "django.utils.html": {
        "escapejs": "thin docstring (which characters is not said)",
        "json_script": "leans on json serialisation",
        "conditional_escape": "depends on an __html__ protocol",
        "format_html": "depends on conditional_escape and str.format",
        "format_html_join": "depends on format_html",
        "linebreaks": "thin docstring (the paragraph rule is not said)",
        "smart_urlquote": "thin docstring",
        "urlize": "no docstring",
        "html_safe": "decorator",
    },
    "django.utils.http": {
        "http_date": "clock (epoch_seconds defaults to now)",
        "parse_http_date_safe": "docstring leans on parse_http_date",
    },
    "django.utils.dateparse": {},
    "requests.utils": {
        "dict_to_sequence": "thin docstring",
        "super_len": "no docstring; IO",
        "get_netrc_auth": "IO (reads netrc)",
        "guess_filename": "object attributes, not data",
        "extract_zipped_paths": "IO (filesystem)",
        "atomic_open": "IO",
        "dict_from_cookiejar": "CookieJar object",
        "add_dict_to_cookiejar": "mutates a CookieJar",
        "get_encodings_from_content": "thin docstring (regexes not said)",
        "stream_decode_response_unicode": "IO stream",
        "iter_slices": "thin docstring",
        "get_unicode_from_response": "response object; IO",
        "is_ipv4_address": "thin docstring",
        "is_valid_cidr": "thin docstring",
        "set_environ": "mutates the environment",
        "should_bypass_proxies": "reads the environment",
        "get_environ_proxies": "reads the environment",
        "resolve_proxies": "reads the environment",
        "default_user_agent": "reads package metadata",
        "default_headers": "thin docstring",
        "guess_json_utf": "thin docstring",
        "check_header_validity": "raises; validation only",
        "rewind_body": "IO",
    },
    "humanize.number": {},
    "humanize.filesize": {},
    "humanize.lists": {},
    "humanize.time": {
        "naturaltime": "clock (relative to now)",
        "naturalday": "clock (relative to today)",
        "naturaldate": "clock (relative to today)",
    },
}


def words_for_signature(name: str, fn) -> str:
    try:
        sig = inspect.signature(fn)
    except (TypeError, ValueError):
        return f"`{name}` takes arguments the docstring describes."
    required, optional, star, starstar, kwonly = [], [], None, None, []
    for p in sig.parameters.values():
        if p.kind is p.VAR_POSITIONAL:
            star = p.name
        elif p.kind is p.VAR_KEYWORD:
            starstar = p.name
        elif p.kind is p.KEYWORD_ONLY:
            kwonly.append(p)
        elif p.default is p.empty:
            required.append(p.name)
        else:
            optional.append(p)

    def default_words(p) -> str:
        d = p.default
        if d is None:
            return f"`{p.name}` (default none)"
        if isinstance(d, (bool, int, float, str, tuple, frozenset)):
            return f"`{p.name}` (default `{d!r}`)"
        if inspect.isclass(d):
            return f"`{p.name}` (default the `{d.__name__}` built-in)"
        if inspect.isbuiltin(d) or inspect.isfunction(d):
            return f"`{p.name}` (default the `{getattr(d, '__name__', '?')}` function)"
        return f"`{p.name}` (no default given: a sentinel meaning 'not supplied')"

    parts = []
    if required:
        parts.append("the required argument" + ("s " if len(required) > 1 else " ") + ", ".join(f"`{r}`" for r in required))
    if star:
        parts.append(f"any number of positional arguments `{star}`")
    if optional:
        parts.append("the optional argument" + ("s " if len(optional) > 1 else " ") + ", ".join(default_words(p) for p in optional))
    if kwonly:
        parts.append("the keyword-only argument" + ("s " if len(kwonly) > 1 else " ") + ", ".join(default_words(p) if p.default is not p.empty else f"`{p.name}`" for p in kwonly))
    if starstar:
        parts.append(f"any keyword arguments `{starstar}`")
    ret = ""
    if sig.return_annotation is not sig.empty:
        ann = sig.return_annotation
        shown = ann if isinstance(ann, str) else getattr(ann, '__name__', repr(ann))
        ret = f" It returns a value of type `{shown}`."
    if not parts:
        return f"`{name}` takes no arguments.{ret}"
    return f"`{name}` takes " + "; ".join(parts) + "." + ret


def main() -> None:
    try:
        import django
        from django.conf import settings
        settings.configure()
        django.setup()
    except Exception as e:  # pragma: no cover
        print("django setup failed:", e)
    PROBLEMS.mkdir(parents=True, exist_ok=True)
    selected, skipped = [], []
    versions = {dist: md.version(dist) for _, dist in MODULES.values()}
    for mn, (source, dist) in MODULES.items():
        m = importlib.import_module(mn)
        skip = SKIP.get(mn, {})
        for name, obj in list(vars(m).items()):
            if name.startswith("_"):
                continue
            try:
                is_fn = inspect.isfunction(obj) and obj.__module__ == mn
            except Exception:
                is_fn = False
            if not is_fn:
                if inspect.isclass(obj) and obj.__module__ == mn:
                    skipped.append({"module": mn, "name": name, "reason": "class, not a function"})
                continue
            doc = inspect.getdoc(obj) or ""
            reason = skip.get("*") or skip.get(name)
            if reason is None and not doc.strip():
                reason = "no docstring"
            if reason:
                skipped.append({"module": mn, "name": name, "reason": reason})
                continue
            fid = f"{mn.split('.')[0]}__{mn.split('.')[-1]}__{name}" if mn.count(".") else f"{mn}__{name}"
            sig_words = words_for_signature(name, obj)
            rec = {
                "id": fid, "source": source, "module": mn, "name": name,
                "qualname": f"{mn}.{name}", "version": versions[dist],
                "signature_words": sig_words, "doc_chars": len(doc),
                "has_doctest": ">>>" in doc,
            }
            selected.append(rec)
            problem = (
                f"{rec['qualname']} (from {source} {rec['version']})\n\n"
                f"Signature, in words: {sig_words}\n\n"
                f"The docstring:\n\n{doc.strip()}\n"
            )
            (PROBLEMS / f"{fid}.md").write_text(problem, encoding="utf-8")
    with open(HERE / "functions.jsonl", "w", encoding="utf-8") as f:
        for r in selected:
            f.write(json.dumps(r) + "\n")
    with open(HERE / "skipped.jsonl", "w", encoding="utf-8") as f:
        for r in skipped:
            f.write(json.dumps(r) + "\n")
    by_source: dict[str, int] = {}
    for r in selected:
        by_source[r["source"]] = by_source.get(r["source"], 0) + 1
    print("selected", len(selected), by_source)
    print("skipped", len(skipped))


if __name__ == "__main__":
    main()
