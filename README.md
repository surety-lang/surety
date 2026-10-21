# Surety

Surety is an obligation language plus a checker plus a graph. Humans write obligations: types, properties, examples. A model writes performances, implementations in a target language; Python is the first target. The checker verifies a performance against its obligation and pins the evidence to both.

What is here, after four slices:

- The parser, the canonical form and its hash, and the renderer that prints a file back from its syntax tree.
- The checker: examples, properties over the examples and over generated inputs, determinism by running twice, totality by a time cap, the boundary validator with refinements, the frame rule, and the underdetermination check, a library of wrong performances run against the obligation alone plus a search for inputs with two passing outputs. Every report opens with a verdict and names whose problem each finding is.
- The behavioural diff between two performances of one obligation.
- The graph: definitions by canonical hash under `graph/`, performances by target and structural hash, one evidence record per performance that a small kernel admits only when its claims are backed by recorded runs, and the queries a reader or a model uses instead of reading files. The corpus is pinned there; `surety verify --rerun` reproduces it.
- Author tooling: a language server and a VS Code extension under `editors/vscode/` (errors as you type, names that resolve, the verdict on save), and `experiments/draft.py`, which drafts an obligation's sentences and then its property lines, never its examples.

```
surety parse <file.sur>...                parse and check (siblings in the directory form the graph)
surety hash <file.sur>                    canonical form, hash, and pinned prose
surety check <file.sur> [<perf.py>]       the examples, the properties over examples and generated inputs,
                                          determinism, totality, the mutant verdict, and the evidence record
surety mutants <file.sur>...              the underdetermination check against the obligation alone
surety diff <file.sur> <old.py> <new.py>  the inputs on which two performances differ, classed against the obligation
surety pin <file.sur> [<perf.py>]         check and pin the definition into graph/
surety show <name | hash>                 a node: its state, its prose, its performances, one hop each way
surety status                             every name in the graph with its state; stale where a source moved on
surety verify [--rerun]                   the kernel over every record; with --rerun, the check again, byte for byte
surety render <file.sur>                  the file printed back from its syntax tree
surety lsp                                the language server over stdio
surety extract <markdown> <out-dir>       cut obligations out of the authored markdown
```

The eleven corpus obligations and their hand-written performances live under `crates/surety/tests/corpus/`; the acceptance tests under `crates/surety/tests/` are the definition of done for each slice. `experiments/closed_loop.py` hands the obligations to a model and checks what it writes; its runs are under `experiments/closed-loop/runs/`.

Follow along at the [surety-lang](https://github.com/surety-lang) organization on GitHub.
