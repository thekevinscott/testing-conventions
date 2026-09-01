### Nested Python test files are excluded from mutation

**Summary**

The bundled cosmic-ray adapter renders `excluded-modules` from a fixed pattern list, and cosmic-ray
expands each entry with `glob.glob(pattern, recursive=True)` from the scanned path. The shipped
patterns carried no `**/` segment, so `*_test.py` reached the scanned path's top level and nothing
below it. A package whose suite lives beside its submodules had every one of those test files
mutated as if it were production source. The patterns are now `**/*_test.py`, `**/test_*.py`, and
`**/conftest.py`.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

A whole-tree Python mutation run (`unit mutation --language python`, no `--base`) over a package
with nested test files reports fewer mutants and, in most trees, fewer survivors: the mutants that
were generated inside test files are gone, along with the survivors among them. Two consequences
are worth expecting. A run that previously failed on survivors pointing at your own `*_test.py`
files now passes, since those mutants are no longer produced. And the reported mutant count drops
by however many mutants your nested suites contributed — in this repo's own Python package, from
1160 to 176.

A diff-scoped run (`--base <ref>`) is unaffected: it already filtered changed files to production
sources before handing them to the engine.

**Verification**

Run the gate over a package whose test files sit below the scanned path:

```
testing-conventions unit mutation --language python src
```

Every reported survivor names a production source. No line of the output names a `*_test.py`,
`test_*.py`, or `conftest.py` path.
