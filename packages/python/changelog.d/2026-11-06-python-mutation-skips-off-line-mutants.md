**Changed** The Python mutation adapter takes repeatable `--lines <file>:<start>-<end>` ranges and
marks every mutant off them skipped before cosmic-ray executes, the way it already skips a mutant
inside an annotation. A diff-scoped run's time follows the changed lines, not the size of the
modules they sit in. Reported results are unchanged.
