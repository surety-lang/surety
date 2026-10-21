# Closed-loop run 20261021-153427

model `claude-opus-5-5`, effort `high`, rounds 1
checker commit `ad29c5e`, binary `target/release/surety` built 2026-10-21 15:33:53

| Obligation | Result | Round | Pass | Fail | Error | Boundary | Runs | Findings | Determinism | Imports | Lines | Seconds | Tokens in/out | First killer |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| p1 | pass | 1 | 5 | 0 | 0 | 0 | 502 | 0 | 502/502 | none | 5 | 4.6 | 261/186 |  |
| p2 | pass | 1 | 6 | 0 | 0 | 0 | 502 | 0 | 502/502 | none | 2 | 5.1 | 334/44 |  |
| p3 | pass | 1 | 11 | 0 | 0 | 0 | 502 | 0 | 502/502 | none | 18 | 3.5 | 907/186 |  |
| p4 | pass | 1 | 9 | 0 | 0 | 0 | 505 | 0 | 505/505 | none | 3 | 6.2 | 678/272 |  |
| p5a | pass | 1 | 8 | 0 | 0 | 0 | 503 | 0 | 503/503 | none | 18 | 4.2 | 821/323 |  |
| p5b | pass | 1 | 8 | 0 | 0 | 0 | 503 | 0 | 503/503 | none | 10 | 1.9 | 596/105 |  |
| p6 | pass | 1 | 9 | 0 | 0 | 0 | 502 | 0 | 502/502 | none | 13 | 5.1 | 638/127 |  |
| p7 | pass | 1 | 9 | 0 | 0 | 0 | 503 | 0 | 503/503 | none | 15 | 4.2 | 1165/160 |  |
| p8 | pass | 1 | 19 | 0 | 0 | 0 | 512 | 0 | 512/512 | none | 55 | 7.6 | 1371/771 |  |
| apply_diff | pass | 1 | 23 | 0 | 0 | 0 | 508 | 0 | 508/508 | none | 47 | 8.5 | 3526/823 |  |
| p14 | pass | 1 | 21 | 0 | 0 | 0 | 503 | 0 | 503/503 | none | 25 | 3.7 | 3376/315 |  |

11 of 11 obligations passed.
