**Fixed** The Python mutation adapter excludes every `.py` under a `tests/` directly beneath the
scanned path from cosmic-ray, whatever its name. A shared helper such as
`tests/integration/served.py` was mutated, and since the suite runs with `--ignore=tests`, every
mutant survived.
