### A Rust subprocess seam owes no whole-body mutant

**Summary**

`unit mutation --language rust` no longer reports cargo-mutants' whole-body replacement — genre
`FnValue`, the `replace f -> T with …` mutants — on a **subprocess seam**: a branch-free function
that runs a `std::process::Command`, either directly or by delegating to another function in the
same file that does. Nothing else changes. Every other genre, and every function that is not a
seam, is judged exactly as before.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

A pull request touching a subprocess wrapper no longer has to account for two to four survivors per
wrapper. If such a wrapper carries a line-scoped `[[rust.exempt]] rules = ["mutation"]` entry, and
the whole-body mutant was the only reason for it, that entry is now stale — and a stale exempt entry
is an error, not a silent pass. Delete it.

A function that both decides and spawns is unchanged: it keeps its whole-body mutant. The route to
green is the split, not an exemption — a pure function returning the argv, asserted by the unit
suite, and a branch-free seam that runs it:

```rust
fn probe_argv(verbose: bool) -> Vec<OsString> { … }   // asserted at the unit tier

fn probe(verbose: bool) -> Result<Output> {           // a seam: no branch, one spawn
    Command::new("probe").args(probe_argv(verbose)).output()
}
```

**Verification**

Run the check over a crate whose only survivors were subprocess wrappers:

```sh
npx testing-conventions unit mutation --language rust .
```

It exits 0 and states its count. A wrapper that still holds a decision names the mutation the
suite has to kill and exits 1:

```
src/probe.rs:8: replace > with == in probe
```
