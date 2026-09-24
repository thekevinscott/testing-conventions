### Rust coverage stops measuring the entry point it tells you to write

**Summary**

`colocated-test` documents two shapes for a Rust binary root, both of which move the logic into the
library behind a `#[cfg(not(test))]` gate. The unit tier runs `cargo llvm-cov --lib --bins`, which
sets `cfg(test)`, so no test can execute a gated item. The `--bins` half nevertheless links the
library a second time as a plain dependency of the binary target's test harness, where `cfg(test)` is
unset: the gated item is compiled and instrumented there, nothing calls it, and it lands in the
report as 0-hit regions, lines and functions.

Whether it landed at all depended on how the linker partitioned that build. `codegen-units = 1`
exposed it, and so did `CARGO_INCREMENTAL=0` at default codegen units — which is what
`Swatinem/rust-cache` sets. The same source read 100% on one machine and 94% in CI, and neither of
the two documented shapes was reliably clean.

The adapter now subtracts a gated item's regions, lines, functions and branches from the ratios it
judges, whole-tree and changed-line alike, reusing the predicate `mutation` already uses. The
exclusion is item-level, so `entrypoint.rs` holding a gated `main` beside a tested `report` keeps
measuring `report`. A gate a test build can still satisfy is not a gate: `#[cfg(any(not(test),
unix))]` compiles under `cargo test`, so it stays a subject.

**Required changes**

_None._ Nothing to configure, and no exemption to write — the shape that used to need a `coverage`
exemption now needs none. An exemption you added for a gated entry point can be deleted.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A crate with a `#[cfg(not(test))]` entry point goes from failing `unit coverage` to passing it on
the machines where the counters were reaching the report, with no source change. Reported totals
drop by the gated items' contribution, so the denominator shrinks and the percentage rises.

**Verification**

From the package root of a crate whose `main` is gated:

```sh
npx testing-conventions unit coverage --language rust .
```

It exits 0 against a 100% floor, and the gated function does not appear as a miss.
