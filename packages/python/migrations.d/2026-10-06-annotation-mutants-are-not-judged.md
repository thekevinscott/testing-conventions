### Python `unit mutation` does not judge a mutant inside a type annotation

**Summary**

cosmic-ray matches its operators over a token tree, where a type annotation and an expression look
alike: `def f(x: dict | None) -> int | None` carries two `|` operators, and the engine rewrites
each of them eleven ways. An annotation is type metadata the interpreter need never evaluate —
`from __future__ import annotations` (PEP 563) keeps it a string, and from Python 3.14 on every
annotation is lazy (PEP 649) — so none of those mutants runs, at import or at call, and no test
can fail on one. The check reported each as a survivor and failed on code whose behavior the suite
pinned completely.

The adapter now collects each annotation's own source span from the module's AST and skips every
mutation site inside one, so cosmic-ray never schedules a suite run for a verdict that was
predetermined. The span covers a parameter annotation, a return annotation, an `AnnAssign`
annotation, and a PEP 695 `type` alias's value.

**Required changes**

_None._ A `uses:` call or a direct
`testing-conventions unit mutation --language python <source>` needs no edit. A line-scoped
`[[python.exempt]] rules = ["mutation"]` entry written to waive an annotation survivor can be
deleted; a `lines` entry whose mutants are now all dropped reads as an unused exemption.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A Python diff that touches only annotations now reports `unit mutation: the engine found no
mutants to test` and passes, where it previously listed one survivor per operator per annotation
and failed. A run over a tree with annotated signatures also takes less wall clock, since a
skipped mutant costs no suite run. Nothing else is dropped: a parameter default, a subscript
value, and a `cast(int | None, x)` argument are live code and keep every mutant they have.

**Verification**

Run the gate over a Python package whose annotations use `|` and whose behavior its colocated
suite pins:

```console
$ testing-conventions unit mutation --language python src
unit mutation: no surviving mutants — every mutation was caught (43 mutant(s) tested)
```

The count covers the bodies and the defaults; no survivor names an annotation.
