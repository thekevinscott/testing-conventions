### A Python survivor names its replacement

**Summary**

The normalized mutant schema the Rust core gates on carries an optional `replacement`, and the
survivor line renders it as `<mutator> (-> <replacement>)`. The TypeScript adapter filled it from
Stryker's own report; the Python adapter left it unset, so a Python survivor read
`calc.py:2: core/ReplaceBinaryOperator_Add_RShift` and a consumer had to know the cosmic-ray
operator taxonomy — and, on a line holding two `+`, which occurrence — before they could write the
assertion that kills it. cosmic-ray records a unified diff of every mutation it ran, so the adapter
now reads the mutated line from that diff and reports it as the replacement.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

A failing `unit mutation --language python` run lists each survivor with the mutated source
appended:

```
before  calc.py:2: core/ReplaceBinaryOperator_Add_RShift
after   calc.py:2: core/ReplaceBinaryOperator_Add_RShift (-> return a >> b)
```

Exit codes, the survivor set, and the exemption surface are unchanged — the line's text is what
moved. A mutation whose diff adds no line, such as a removed decorator, is listed by its operator
name as before.

**Verification**

Run the gate over a package with a surviving mutant:

```
testing-conventions unit mutation --language python src
```

Every listed survivor whose mutation replaced source carries a `(-> …)` naming the mutated line.
