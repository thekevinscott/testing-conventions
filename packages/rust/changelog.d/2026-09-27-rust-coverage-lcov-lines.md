**Fixed** **The Rust line floor reads a view that can name its missing lines** (#743). A `--json`
export's `totals` sum each instantiation group's own line tally, and `--lib --bins` compiles the
library twice, so a line both groups map counted twice — once covered, once not, when only one
group ran it. A file could sit one line short of a 100% floor while `llvm-cov show`, `--text`,
`--show-missing-lines` and `lcov` all reported it fully covered, leaving no line to write a test
for.

`unit coverage --language rust` now takes its line count and covered count from the `DA:` records
of an `lcov` export, which llvm-cov writes through the merged line view those other commands read.
Both exports come from one run: the check re-reads the profile the run left behind rather than
running the suite a second time. Regions, functions and branches still come from the `--json`
totals, where the per-function detail the gated-item subtraction needs lives.

Line denominators drop on adoption — this crate's fell from 11619 to 11251 — because a line two
groups map now counts once. A percentage that was passing does not start failing: every line the
change removes from the denominator is a duplicate of one still in it.
