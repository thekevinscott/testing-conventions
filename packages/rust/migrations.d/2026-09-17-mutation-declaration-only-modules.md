### Declaration-only modules are not mutation subjects

**Summary**

`unit mutation` mutated every changed line inside a declaration-only module (a constants file, a
re-export barrel, a type-only file) exactly like any other file, so a survivor there demanded a
line-scoped exemption restating "this file has no logic" — the mutation analog of the
colocated-test gap the declaration-only-modules change closed. The same subject predicate —
`colocated_test::Language::is_subject`, "a function or control flow anywhere in the file" — now
also gates mutation, across all three languages. Its Rust arm, stubbed to `false` and matched by
no caller, becomes real: "a file with a `fn` anywhere," Rust's structural equivalent (no
module-level control flow).

**Required changes**

_None._ A `mutation` exemption naming a line inside a declaration-only module can be deleted; it
now lifts nothing that was ever flagged.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A declaration-only module's mutants are dropped before judging, so it never contributes a
survivor. TypeScript additionally short-circuits: a diff touching only declaration-only modules
never invokes Stryker, reporting `unit mutation: no mutatable changed lines — engine not run`
instead of running the engine over an empty mutate scope. Python and Rust still run the engine
when a declaration-only module is the only change (a `TIMEOUT` constant, say) but report `unit
mutation: the engine found no mutants to test` once its mutants are dropped.

`unit colocated-test`'s own behavior is unchanged for every language: Rust presence there is
still decided by its own inline-test walk, not this predicate. What changes is that
`is_subject`'s Rust arm — previously a hardcoded `false` no caller exercised — now returns real
answers for its new caller, `unit mutation`.

**Verification**

```console
$ testing-conventions unit mutation --language rust <crate>
```

A crate whose only diff-scoped change is a declaration-only module's constant reports `unit
mutation: the engine found no mutants to test`, not a survivor.
