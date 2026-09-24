### Rust mutation stops failing the entry point it tells you to write

**Summary**

`colocated-test` documents two shapes for a Rust binary root, both of which move the logic into the
library behind a `#[cfg(not(test))]` gate. `colocated-test`, `unit lint` and `unit coverage` all pass
on that shape. `unit mutation` then failed on it: cargo-mutants reads the source rather than the
build, so it mutates a gated function, and the unit tier's `--lib --bins` sets `cfg(test)` — the
mutated code is not in the binary the suite runs, no test can reach it, and every such mutant is
reported MISSED. The conventions prescribed a shape and the next gate punished it.

The Rust adapter now drops those mutants before judging, the same way it already drops a mutant in a
declaration-only module. The filter is item-level, not file-level, so `entrypoint.rs` holding a gated
`main` beside a tested `report` keeps judging `report`'s mutants and loses only `main`'s. A gate a
test build can still satisfy is not a gate: `#[cfg(any(not(test), unix))]` compiles under
`cargo test`, so it stays a mutation subject.

**Required changes**

_None._ Nothing to configure, and no exemption to write — the shape that used to need a `mutation`
exemption now needs none. An exemption you added for a gated entry point can be deleted; a
line-scoped one that now points at a dropped mutant is left alone rather than flagged as
over-exemption.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A crate with a `#[cfg(not(test))]` entry point goes from failing `unit mutation` to passing it, with
no source change. The reported mutant count drops by the number of gated mutants the engine found,
since a dropped mutant is no longer judged.

**Verification**

From the package root of a crate whose `main` is gated:

```sh
npx testing-conventions unit mutation --language rust .
```

It exits 0 and states the count it judged, with no survivor for the gated function.
