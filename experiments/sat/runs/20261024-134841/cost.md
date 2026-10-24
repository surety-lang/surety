# Cost by family

Cost is lines of the performance's own source executed (a `sys.settrace` count in the child process),
fitted as log(lines) against size (exponential: the base per unit of size) and against log(size)
(polynomial: the exponent). Both fits are given with R^2; the range is the sizes the fit covers.
A fit over fewer than three sizes is not reported. Timed out means past the checker's 10 s cap.

| Family | Size means | Performance | Fitted range (points) | Exponential: base per unit (R^2) | Polynomial: exponent (R^2) | Better fit | Timed out at | Skipped at |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| random3sat | n, the variable count (clauses = round(4.26 n)) | plain | 10..24 (22) | 1.919 (0.89) | 10.30 (0.91) | polynomial | 24, 30 | 40, 50, 75, 100, 150, 200 |
| random3sat | n, the variable count (clauses = round(4.26 n)) | dpll | 10..100 (39) | 1.101 (0.85) | 3.78 (0.89) | polynomial | 150 | 200 |
| random3sat | n, the variable count (clauses = round(4.26 n)) | cdcl | 10..200 (45) | 1.046 (0.92) | 2.68 (0.90) | exponential | - | - |
| php | n, the hole count (n + 1 pigeons, n(n+1) variables) | plain | 3..4 (2) | - | - | fewer than three sizes completed; no fit | 5 | 6, 7, 8, 9, 10, 11, 12 |
| php | n, the hole count (n + 1 pigeons, n(n+1) variables) | dpll | 3..8 (6) | 6.482 (1.00) | 9.43 (0.97) | exponential | 9 | 10, 11, 12 |
| php | n, the hole count (n + 1 pigeons, n(n+1) variables) | cdcl | 3..7 (5) | 10.456 (0.99) | 10.87 (0.95) | exponential | 8 | 9, 10, 11, 12 |
| tseitin | the vertex count of the 3-regular graph (3/2 as many variables); odd charge, unsat | plain | 6..14 (5) | 2.831 (1.00) | 9.76 (0.98) | exponential | 16 | 20, 24, 30, 40 |
| tseitin | the vertex count of the 3-regular graph (3/2 as many variables); odd charge, unsat | dpll | 6..30 (9) | 1.474 (0.99) | 5.69 (0.94) | exponential | 40 | - |
| tseitin | the vertex count of the 3-regular graph (3/2 as many variables); odd charge, unsat | cdcl | 6..40 (10) | 1.215 (0.93) | 3.49 (0.92) | exponential | - | - |
| factoring | the width of each factor in bits (the semiprime has twice as many, less one) | plain | none | - | - | fewer than three sizes completed; no fit | 5 | 6, 7, 8, 9, 10, 11, 12 |
| factoring | the width of each factor in bits (the semiprime has twice as many, less one) | dpll | 5..12 (8) | 1.956 (0.71) | 5.45 (0.72) | polynomial | - | - |
| factoring | the width of each factor in bits (the semiprime has twice as many, less one) | cdcl | 5..12 (8) | 1.617 (0.47) | 3.93 (0.48) | polynomial | - | - |

## The canaries

- PHP_n, plain: fewer than three sizes completed; no fit; sizes completed [3, 4]; timed out at [5].
- PHP_n, dpll: base 6.482 per hole (R^2 1.00), fitted on n = 3..8; polynomial exponent 9.43 (R^2 0.97); timed out at 9.
- PHP_n, cdcl: base 10.456 per hole (R^2 0.99), fitted on n = 3..7; polynomial exponent 10.87 (R^2 0.95); timed out at 8.
- Factoring, plain: no instance solved within the cap.
- Factoring, dpll: largest semiprime solved within the cap N = 5094443 = 2113 x 2411 (23 bits, factors 12 bits; 432 variables, 2220 clauses; 6769078 lines, 0.1399 s). Timed out at bits none.
- Factoring, cdcl: largest semiprime solved within the cap N = 5094443 = 2113 x 2411 (23 bits, factors 12 bits; 432 variables, 2220 clauses; 758828 lines, 0.02 s). Timed out at bits none.

## Every instance

| Family | Size | # | Vars | Clauses | Performance | Status | Answer | Wall s | Lines | Verification | Note |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| random3sat | 10 | 0 | 10 | 43 | plain | ok | sat | 0.000 | 2665 | model verified |  |
| random3sat | 10 | 0 | 10 | 43 | dpll | ok | sat | 0.000 | 5114 | model verified |  |
| random3sat | 10 | 0 | 10 | 43 | cdcl | ok | sat | 0.000 | 3321 | model verified |  |
| random3sat | 10 | 1 | 10 | 43 | plain | ok | sat | 0.001 | 52007 | model verified |  |
| random3sat | 10 | 1 | 10 | 43 | dpll | ok | sat | 0.000 | 5220 | model verified |  |
| random3sat | 10 | 1 | 10 | 43 | cdcl | ok | sat | 0.000 | 4946 | model verified |  |
| random3sat | 10 | 2 | 10 | 43 | plain | ok | sat | 0.002 | 72119 | model verified |  |
| random3sat | 10 | 2 | 10 | 43 | dpll | ok | sat | 0.000 | 5716 | model verified |  |
| random3sat | 10 | 2 | 10 | 43 | cdcl | ok | sat | 0.000 | 7252 | model verified |  |
| random3sat | 12 | 0 | 12 | 51 | plain | ok | sat | 0.006 | 295598 | model verified |  |
| random3sat | 12 | 0 | 12 | 51 | dpll | ok | sat | 0.000 | 6316 | model verified |  |
| random3sat | 12 | 0 | 12 | 51 | cdcl | ok | sat | 0.000 | 6828 | model verified |  |
| random3sat | 12 | 1 | 12 | 51 | plain | ok | sat | 0.001 | 45314 | model verified |  |
| random3sat | 12 | 1 | 12 | 51 | dpll | ok | sat | 0.000 | 6528 | model verified |  |
| random3sat | 12 | 1 | 12 | 51 | cdcl | ok | sat | 0.000 | 4282 | model verified |  |
| random3sat | 12 | 2 | 12 | 51 | plain | ok | sat | 0.003 | 141648 | model verified |  |
| random3sat | 12 | 2 | 12 | 51 | dpll | ok | sat | 0.001 | 19711 | model verified |  |
| random3sat | 12 | 2 | 12 | 51 | cdcl | ok | sat | 0.000 | 5350 | model verified |  |
| random3sat | 14 | 0 | 14 | 60 | plain | ok | unsat | 0.033 | 1473963 | unsat by exhaustion (the oracle) |  |
| random3sat | 14 | 0 | 14 | 60 | dpll | ok | unsat | 0.001 | 26436 | unsat, agrees with plain |  |
| random3sat | 14 | 0 | 14 | 60 | cdcl | ok | unsat | 0.000 | 7628 | unsat, agrees with plain |  |
| random3sat | 14 | 1 | 14 | 60 | plain | ok | sat | 0.012 | 580239 | model verified |  |
| random3sat | 14 | 1 | 14 | 60 | dpll | ok | sat | 0.000 | 7784 | model verified |  |
| random3sat | 14 | 1 | 14 | 60 | cdcl | ok | sat | 0.000 | 6021 | model verified |  |
| random3sat | 14 | 2 | 14 | 60 | plain | ok | sat | 0.018 | 623591 | model verified |  |
| random3sat | 14 | 2 | 14 | 60 | dpll | ok | sat | 0.001 | 17805 | model verified |  |
| random3sat | 14 | 2 | 14 | 60 | cdcl | ok | sat | 0.000 | 5388 | model verified |  |
| random3sat | 16 | 0 | 16 | 68 | plain | ok | unsat | 0.167 | 5933661 | unsat by exhaustion (the oracle) |  |
| random3sat | 16 | 0 | 16 | 68 | dpll | ok | unsat | 0.001 | 28668 | unsat, agrees with plain |  |
| random3sat | 16 | 0 | 16 | 68 | cdcl | ok | unsat | 0.000 | 10779 | unsat, agrees with plain |  |
| random3sat | 16 | 1 | 16 | 68 | plain | ok | sat | 0.043 | 1880883 | model verified |  |
| random3sat | 16 | 1 | 16 | 68 | dpll | ok | sat | 0.000 | 8985 | model verified |  |
| random3sat | 16 | 1 | 16 | 68 | cdcl | ok | sat | 0.000 | 9432 | model verified |  |
| random3sat | 16 | 2 | 16 | 68 | plain | ok | sat | 0.073 | 3152330 | model verified |  |
| random3sat | 16 | 2 | 16 | 68 | dpll | ok | sat | 0.001 | 23972 | model verified |  |
| random3sat | 16 | 2 | 16 | 68 | cdcl | ok | sat | 0.000 | 11255 | model verified |  |
| random3sat | 18 | 0 | 18 | 77 | plain | ok | sat | 0.074 | 3025213 | model verified |  |
| random3sat | 18 | 0 | 18 | 77 | dpll | ok | sat | 0.000 | 12548 | model verified |  |
| random3sat | 18 | 0 | 18 | 77 | cdcl | ok | sat | 0.000 | 8353 | model verified |  |
| random3sat | 18 | 1 | 18 | 77 | plain | ok | sat | 0.307 | 11898996 | model verified |  |
| random3sat | 18 | 1 | 18 | 77 | dpll | ok | sat | 0.001 | 21119 | model verified |  |
| random3sat | 18 | 1 | 18 | 77 | cdcl | ok | sat | 0.000 | 15941 | model verified |  |
| random3sat | 18 | 2 | 18 | 77 | plain | ok | sat | 0.251 | 11003980 | model verified |  |
| random3sat | 18 | 2 | 18 | 77 | dpll | ok | sat | 0.000 | 16328 | model verified |  |
| random3sat | 18 | 2 | 18 | 77 | cdcl | ok | sat | 0.001 | 18179 | model verified |  |
| random3sat | 20 | 0 | 20 | 85 | plain | ok | sat | 1.024 | 48600062 | model verified |  |
| random3sat | 20 | 0 | 20 | 85 | dpll | ok | sat | 0.000 | 13106 | model verified |  |
| random3sat | 20 | 0 | 20 | 85 | cdcl | ok | sat | 0.001 | 14780 | model verified |  |
| random3sat | 20 | 1 | 20 | 85 | plain | ok | unsat | 2.231 | 91992496 | unsat by exhaustion (the oracle) |  |
| random3sat | 20 | 1 | 20 | 85 | dpll | ok | unsat | 0.002 | 71148 | unsat, agrees with plain |  |
| random3sat | 20 | 1 | 20 | 85 | cdcl | ok | unsat | 0.000 | 13054 | unsat, agrees with plain |  |
| random3sat | 20 | 2 | 20 | 85 | plain | ok | sat | 0.256 | 10656115 | model verified |  |
| random3sat | 20 | 2 | 20 | 85 | dpll | ok | sat | 0.002 | 84150 | model verified |  |
| random3sat | 20 | 2 | 20 | 85 | cdcl | ok | sat | 0.000 | 10641 | model verified |  |
| random3sat | 22 | 0 | 22 | 94 | plain | ok | unsat | 7.962 | 368664776 | unsat by exhaustion (the oracle) |  |
| random3sat | 22 | 0 | 22 | 94 | dpll | ok | unsat | 0.003 | 135340 | unsat, agrees with plain |  |
| random3sat | 22 | 0 | 22 | 94 | cdcl | ok | unsat | 0.001 | 14485 | unsat, agrees with plain |  |
| random3sat | 22 | 1 | 22 | 94 | plain | ok | sat | 0.698 | 33340611 | model verified |  |
| random3sat | 22 | 1 | 22 | 94 | dpll | ok | sat | 0.001 | 32443 | model verified |  |
| random3sat | 22 | 1 | 22 | 94 | cdcl | ok | sat | 0.000 | 10768 | model verified |  |
| random3sat | 22 | 2 | 22 | 94 | plain | ok | sat | 0.750 | 29999319 | model verified |  |
| random3sat | 22 | 2 | 22 | 94 | dpll | ok | sat | 0.001 | 30467 | model verified |  |
| random3sat | 22 | 2 | 22 | 94 | cdcl | ok | sat | 0.000 | 7080 | model verified |  |
| random3sat | 24 | 0 | 24 | 102 | plain | timeout | - | - | - |  |  |
| random3sat | 24 | 0 | 24 | 102 | dpll | ok | sat | 0.001 | 26973 | model verified |  |
| random3sat | 24 | 0 | 24 | 102 | cdcl | ok | sat | 0.001 | 17877 | model verified |  |
| random3sat | 24 | 1 | 24 | 102 | plain | timeout | - | - | - |  |  |
| random3sat | 24 | 1 | 24 | 102 | dpll | ok | unsat | 0.002 | 104604 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 24 | 1 | 24 | 102 | cdcl | ok | unsat | 0.001 | 25922 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 24 | 2 | 24 | 102 | plain | ok | sat | 2.720 | 122533952 | model verified |  |
| random3sat | 24 | 2 | 24 | 102 | dpll | ok | sat | 0.001 | 42557 | model verified |  |
| random3sat | 24 | 2 | 24 | 102 | cdcl | ok | sat | 0.000 | 7623 | model verified |  |
| random3sat | 30 | 0 | 30 | 128 | plain | timeout | - | - | - |  |  |
| random3sat | 30 | 0 | 30 | 128 | dpll | ok | sat | 0.003 | 119786 | model verified |  |
| random3sat | 30 | 0 | 30 | 128 | cdcl | ok | sat | 0.001 | 25053 | model verified |  |
| random3sat | 30 | 1 | 30 | 128 | plain | timeout | - | - | - |  |  |
| random3sat | 30 | 1 | 30 | 128 | dpll | ok | sat | 0.004 | 157664 | model verified |  |
| random3sat | 30 | 1 | 30 | 128 | cdcl | ok | sat | 0.001 | 25422 | model verified |  |
| random3sat | 30 | 2 | 30 | 128 | plain | timeout | - | - | - |  |  |
| random3sat | 30 | 2 | 30 | 128 | dpll | ok | unsat | 0.006 | 268746 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 30 | 2 | 30 | 128 | cdcl | ok | unsat | 0.001 | 55215 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 40 | 0 | 40 | 170 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 40 | 0 | 40 | 170 | dpll | ok | unsat | 0.024 | 961049 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 40 | 0 | 40 | 170 | cdcl | ok | unsat | 0.002 | 70010 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 40 | 1 | 40 | 170 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 40 | 1 | 40 | 170 | dpll | ok | unsat | 0.021 | 960291 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 40 | 1 | 40 | 170 | cdcl | ok | unsat | 0.001 | 45946 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 40 | 2 | 40 | 170 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 40 | 2 | 40 | 170 | dpll | ok | sat | 0.004 | 116924 | model verified |  |
| random3sat | 40 | 2 | 40 | 170 | cdcl | ok | sat | 0.002 | 80658 | model verified |  |
| random3sat | 50 | 0 | 50 | 213 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 50 | 0 | 50 | 213 | dpll | ok | unsat | 0.059 | 2729769 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 50 | 0 | 50 | 213 | cdcl | ok | unsat | 0.003 | 125049 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 50 | 1 | 50 | 213 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 50 | 1 | 50 | 213 | dpll | ok | unsat | 0.047 | 1793686 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 50 | 1 | 50 | 213 | cdcl | ok | unsat | 0.003 | 135505 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 50 | 2 | 50 | 213 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 50 | 2 | 50 | 213 | dpll | ok | unsat | 0.081 | 3025882 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 50 | 2 | 50 | 213 | cdcl | ok | unsat | 0.003 | 148577 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 75 | 0 | 75 | 320 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 75 | 0 | 75 | 320 | dpll | ok | sat | 0.009 | 418024 | model verified |  |
| random3sat | 75 | 0 | 75 | 320 | cdcl | ok | sat | 0.004 | 131961 | model verified |  |
| random3sat | 75 | 1 | 75 | 320 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 75 | 1 | 75 | 320 | dpll | ok | unsat | 0.469 | 19605562 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 75 | 1 | 75 | 320 | cdcl | ok | unsat | 0.009 | 410165 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 75 | 2 | 75 | 320 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 75 | 2 | 75 | 320 | dpll | ok | unsat | 0.757 | 30998236 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 75 | 2 | 75 | 320 | cdcl | ok | unsat | 0.016 | 585805 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 100 | 0 | 100 | 426 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 100 | 0 | 100 | 426 | dpll | ok | sat | 1.697 | 77371700 | model verified |  |
| random3sat | 100 | 0 | 100 | 426 | cdcl | ok | sat | 0.009 | 373266 | model verified |  |
| random3sat | 100 | 1 | 100 | 426 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 100 | 1 | 100 | 426 | dpll | ok | sat | 0.922 | 38351463 | model verified |  |
| random3sat | 100 | 1 | 100 | 426 | cdcl | ok | sat | 0.013 | 530983 | model verified |  |
| random3sat | 100 | 2 | 100 | 426 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 100 | 2 | 100 | 426 | dpll | ok | sat | 0.183 | 6233876 | model verified |  |
| random3sat | 100 | 2 | 100 | 426 | cdcl | ok | sat | 0.002 | 80776 | model verified |  |
| random3sat | 150 | 0 | 150 | 639 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 150 | 0 | 150 | 639 | dpll | timeout | - | - | - |  |  |
| random3sat | 150 | 0 | 150 | 639 | cdcl | ok | sat | 0.069 | 3011898 | model verified |  |
| random3sat | 150 | 1 | 150 | 639 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 150 | 1 | 150 | 639 | dpll | timeout | - | - | - |  |  |
| random3sat | 150 | 1 | 150 | 639 | cdcl | ok | unsat | 0.372 | 12983661 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 150 | 2 | 150 | 639 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 150 | 2 | 150 | 639 | dpll | timeout | - | - | - |  |  |
| random3sat | 150 | 2 | 150 | 639 | cdcl | ok | unsat | 0.261 | 11980468 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| random3sat | 200 | 0 | 200 | 852 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 200 | 0 | 200 | 852 | dpll | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 200 | 0 | 200 | 852 | cdcl | ok | sat | 2.907 | 103180474 | model verified |  |
| random3sat | 200 | 1 | 200 | 852 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 200 | 1 | 200 | 852 | dpll | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 200 | 1 | 200 | 852 | cdcl | ok | sat | 0.173 | 6748958 | model verified |  |
| random3sat | 200 | 2 | 200 | 852 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 200 | 2 | 200 | 852 | dpll | skipped | - | - | - |  | timed out on every instance of a smaller size |
| random3sat | 200 | 2 | 200 | 852 | cdcl | ok | sat | 0.371 | 15851714 | model verified |  |
| php | 3 | 0 | 12 | 22 | plain | ok | unsat | 0.007 | 248172 | unsat by exhaustion (the oracle) |  |
| php | 3 | 0 | 12 | 22 | dpll | ok | unsat | 0.000 | 5334 | unsat, agrees with plain |  |
| php | 3 | 0 | 12 | 22 | cdcl | ok | unsat | 0.000 | 5450 | unsat, agrees with plain |  |
| php | 4 | 0 | 20 | 45 | plain | ok | unsat | 2.054 | 91055320 | unsat by exhaustion (the oracle) |  |
| php | 4 | 0 | 20 | 45 | dpll | ok | unsat | 0.001 | 30156 | unsat, agrees with plain |  |
| php | 4 | 0 | 20 | 45 | cdcl | ok | unsat | 0.001 | 30344 | unsat, agrees with plain |  |
| php | 5 | 0 | 30 | 81 | plain | timeout | - | - | - |  |  |
| php | 5 | 0 | 30 | 81 | dpll | ok | unsat | 0.003 | 174718 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| php | 5 | 0 | 30 | 81 | cdcl | ok | unsat | 0.007 | 253006 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| php | 6 | 0 | 42 | 133 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 6 | 0 | 42 | 133 | dpll | ok | unsat | 0.021 | 1104759 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| php | 6 | 0 | 42 | 133 | cdcl | ok | unsat | 0.085 | 3633870 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| php | 7 | 0 | 56 | 204 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 7 | 0 | 56 | 204 | dpll | ok | unsat | 0.140 | 7852280 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| php | 7 | 0 | 56 | 204 | cdcl | ok | unsat | 1.292 | 62248946 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| php | 8 | 0 | 72 | 297 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 8 | 0 | 72 | 297 | dpll | ok | unsat | 1.158 | 63047484 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| php | 8 | 0 | 72 | 297 | cdcl | timeout | - | - | - |  |  |
| php | 9 | 0 | 90 | 415 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 9 | 0 | 90 | 415 | dpll | timeout | unsat | 10.703 | - |  |  |
| php | 9 | 0 | 90 | 415 | cdcl | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 10 | 0 | 110 | 561 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 10 | 0 | 110 | 561 | dpll | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 10 | 0 | 110 | 561 | cdcl | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 11 | 0 | 132 | 738 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 11 | 0 | 132 | 738 | dpll | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 11 | 0 | 132 | 738 | cdcl | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 12 | 0 | 156 | 949 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 12 | 0 | 156 | 949 | dpll | skipped | - | - | - |  | timed out on every instance of a smaller size |
| php | 12 | 0 | 156 | 949 | cdcl | skipped | - | - | - |  | timed out on every instance of a smaller size |
| tseitin | 6 | 0 | 9 | 24 | plain | ok | unsat | 0.001 | 37288 | unsat by exhaustion (the oracle) |  |
| tseitin | 6 | 0 | 9 | 24 | dpll | ok | unsat | 0.000 | 8596 | unsat, agrees with plain |  |
| tseitin | 6 | 0 | 9 | 24 | cdcl | ok | unsat | 0.000 | 10775 | unsat, agrees with plain |  |
| tseitin | 8 | 0 | 12 | 32 | plain | ok | unsat | 0.007 | 300156 | unsat by exhaustion (the oracle) |  |
| tseitin | 8 | 0 | 12 | 32 | dpll | ok | unsat | 0.001 | 23559 | unsat, agrees with plain |  |
| tseitin | 8 | 0 | 12 | 32 | cdcl | ok | unsat | 0.001 | 27657 | unsat, agrees with plain |  |
| tseitin | 10 | 0 | 15 | 40 | plain | ok | unsat | 0.050 | 2406336 | unsat by exhaustion (the oracle) |  |
| tseitin | 10 | 0 | 15 | 40 | dpll | ok | unsat | 0.001 | 46890 | unsat, agrees with plain |  |
| tseitin | 10 | 0 | 15 | 40 | cdcl | ok | unsat | 0.001 | 35402 | unsat, agrees with plain |  |
| tseitin | 12 | 0 | 18 | 48 | plain | ok | unsat | 0.455 | 19262948 | unsat by exhaustion (the oracle) |  |
| tseitin | 12 | 0 | 18 | 48 | dpll | ok | unsat | 0.002 | 101069 | unsat, agrees with plain |  |
| tseitin | 12 | 0 | 18 | 48 | cdcl | ok | unsat | 0.002 | 102919 | unsat, agrees with plain |  |
| tseitin | 14 | 0 | 21 | 56 | plain | ok | unsat | 3.161 | 154130888 | unsat by exhaustion (the oracle) |  |
| tseitin | 14 | 0 | 21 | 56 | dpll | ok | unsat | 0.004 | 234608 | unsat, agrees with plain |  |
| tseitin | 14 | 0 | 21 | 56 | cdcl | ok | unsat | 0.001 | 39291 | unsat, agrees with plain |  |
| tseitin | 16 | 0 | 24 | 64 | plain | timeout | - | - | - |  |  |
| tseitin | 16 | 0 | 24 | 64 | dpll | ok | unsat | 0.005 | 205075 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| tseitin | 16 | 0 | 24 | 64 | cdcl | ok | unsat | 0.003 | 93623 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| tseitin | 20 | 0 | 30 | 80 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| tseitin | 20 | 0 | 30 | 80 | dpll | ok | unsat | 0.072 | 3684057 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| tseitin | 20 | 0 | 30 | 80 | cdcl | ok | unsat | 0.015 | 788148 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| tseitin | 24 | 0 | 36 | 96 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| tseitin | 24 | 0 | 36 | 96 | dpll | ok | unsat | 0.263 | 13254943 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| tseitin | 24 | 0 | 36 | 96 | cdcl | ok | unsat | 0.012 | 602259 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| tseitin | 30 | 0 | 45 | 120 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| tseitin | 30 | 0 | 45 | 120 | dpll | ok | unsat | 1.675 | 81728776 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| tseitin | 30 | 0 | 45 | 120 | cdcl | ok | unsat | 0.071 | 3256240 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| tseitin | 40 | 0 | 60 | 160 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| tseitin | 40 | 0 | 60 | 160 | dpll | timeout | - | - | - |  |  |
| tseitin | 40 | 0 | 60 | 160 | cdcl | ok | unsat | 0.126 | 6879095 | unsat, unverified (no certificate; a DRAT check is the next step) |  |
| factoring | 5 | 0 | 75 | 330 | plain | timeout | - | - | - |  |  |
| factoring | 5 | 0 | 75 | 330 | dpll | ok | sat | 0.004 | 178988 | model verified | x=23 y=19 x*y==N |
| factoring | 5 | 0 | 75 | 330 | cdcl | ok | sat | 0.002 | 87797 | model verified | x=19 y=23 x*y==N |
| factoring | 6 | 0 | 108 | 498 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| factoring | 6 | 0 | 108 | 498 | dpll | ok | sat | 0.006 | 230774 | model verified | x=43 y=41 x*y==N |
| factoring | 6 | 0 | 108 | 498 | cdcl | ok | sat | 0.002 | 54043 | model verified | x=41 y=43 x*y==N |
| factoring | 7 | 0 | 147 | 700 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| factoring | 7 | 0 | 147 | 700 | dpll | ok | sat | 0.014 | 680588 | model verified | x=67 y=89 x*y==N |
| factoring | 7 | 0 | 147 | 700 | cdcl | ok | sat | 0.004 | 134303 | model verified | x=89 y=67 x*y==N |
| factoring | 8 | 0 | 192 | 936 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| factoring | 8 | 0 | 192 | 936 | dpll | ok | sat | 0.046 | 2412164 | model verified | x=191 y=229 x*y==N |
| factoring | 8 | 0 | 192 | 936 | cdcl | ok | sat | 0.028 | 973424 | model verified | x=191 y=229 x*y==N |
| factoring | 9 | 0 | 243 | 1206 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| factoring | 9 | 0 | 243 | 1206 | dpll | ok | sat | 0.113 | 5660072 | model verified | x=383 y=389 x*y==N |
| factoring | 9 | 0 | 243 | 1206 | cdcl | ok | sat | 0.030 | 1364885 | model verified | x=383 y=389 x*y==N |
| factoring | 10 | 0 | 300 | 1510 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| factoring | 10 | 0 | 300 | 1510 | dpll | ok | sat | 0.024 | 1254410 | model verified | x=1019 y=769 x*y==N |
| factoring | 10 | 0 | 300 | 1510 | cdcl | ok | sat | 0.004 | 123772 | model verified | x=769 y=1019 x*y==N |
| factoring | 11 | 0 | 363 | 1848 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| factoring | 11 | 0 | 363 | 1848 | dpll | ok | sat | 1.316 | 65521219 | model verified | x=2003 y=1091 x*y==N |
| factoring | 11 | 0 | 363 | 1848 | cdcl | ok | sat | 0.193 | 8278606 | model verified | x=2003 y=1091 x*y==N |
| factoring | 12 | 0 | 432 | 2220 | plain | skipped | - | - | - |  | timed out on every instance of a smaller size |
| factoring | 12 | 0 | 432 | 2220 | dpll | ok | sat | 0.140 | 6769078 | model verified | x=2411 y=2113 x*y==N |
| factoring | 12 | 0 | 432 | 2220 | cdcl | ok | sat | 0.020 | 758828 | model verified | x=2113 y=2411 x*y==N |
