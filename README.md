# Surety

Surety is an obligation language plus a checker plus a graph. Humans write obligations: types, properties, examples. A model writes performances, implementations in a target language; Python is the first target. The checker verifies a performance against its obligation.

Slice 1 is here: the parser, the canonical form and hash, the Python runner, the example checker, and the property evaluator with the frame rule and the boundary validator.

```
surety parse <file.sur>...          parse and check (siblings in the directory form the graph)
surety hash <file.sur>              canonical form, hash, and pinned prose
surety check <file.sur> [<perf.py>] run the examples and properties against a performance
surety extract <markdown> <out-dir> cut obligations out of the authored markdown
```

The eleven corpus obligations and their hand-written performances live under `crates/surety/tests/corpus/`; the acceptance tests under `crates/surety/tests/` are the definition of done for the slice.

Follow along at the [surety-lang](https://github.com/surety-lang) organization on GitHub.
