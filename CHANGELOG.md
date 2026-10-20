# Changelog

## 0.1.0 — 2026-10-20

Slice 1: parse, run, check.

- `surety-syntax`: lexer and parser for the obligation language as the eleven corpus obligations use it; a cross-file graph with type resolution; static checks for the mistakes the brief lists (unbound names, a quantifier mid-line, `where` without a body, two bodies, an unused binder, a list relation across element types, a pattern as a value, `else` without `where`, `=` for `==`, a non-literal `in:`); canonical form and SHA-256 hash with prose pinned separately.
- `surety-py`: runs a Python performance in a subprocess over a line-delimited JSON boundary, one process per module, with a per-call timeout.
- `surety-check`: boundary validator with list refinements (`[0..3]`, `[unique f]`); example checker with `...` filled by the frame rule and multiset comparison where `unordered` says; property evaluator by brute force over the example runs, with graph references into the same module or another obligation's performance; the frame rule as implicit properties.
- `surety`: the `extract`, `parse`, `hash`, and `check` commands; the eleven fixtures, their hand-written performances, fifteen negative fixtures, and seventeen mutant performances as the acceptance suite.
