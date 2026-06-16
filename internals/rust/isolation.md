# Rust — isolation & external-deps (design)

Design pass for [#44](https://github.com/thekevinscott/testing-conventions/issues/44)
(Phase 3, "needs a design pass first"). It resolves the open questions and carves
the work into red→green slices. **No detector ships from this doc** — it is the
spec the implementation slices build against.

Read [testing.md](testing.md) first (inline `#[cfg(test)]`, `Cursor` for in-memory
I/O, integration tests in `tests/`) and the README's **Isolation** (Unit) and
**External Dependencies** (Integration) rules — this doc makes those two rules
deterministic for Rust.

## The rule (restated)

| Kind | Convention | Violation |
| --- | --- | --- |
| **Unit** | An inline `#[cfg(test)] mod tests` exercises only the unit under test — its parent module, reached via `super`. Collaborators are injected as trait doubles (hand-rolled or `mockall`); the compiler checks the double. | A call *out of the test's own module* — into another first-party module, an external crate, or effectful `std`. A glob import of anything but `super`. |
| **Integration** | A `tests/` integration crate runs first-party code for real; only external crates / effectful `std` may be doubled. | Doubling a **first-party** item (e.g. `#[double]` on a `use crate::…` import). |

Both honor an inline `waiver:` escape hatch (see [Waiver](#waiver)).

## Approach: `syn` heuristic now, `dylint` later

`syn` parses Rust to an AST but does **no name resolution** — it cannot tell whether
a bare `foo()` is a local fn, a `super::foo`, or a glob-imported collaborator.
`dylint` runs as a rustc-driver lint with full name/type resolution and *could* be
exact, but it needs a pinned nightly, compiles the target crate, and is heavyweight.

**Decision:** ship a **deterministic `syn` heuristic** that flags the syntactically
visible out-of-module signals (qualified path calls + foreign imports), with
documented precision limits. This matches the repo's lightweight-parser precedent
(`rustpython` for Python in `lint.rs`, `syn` for Rust here) and the bright-line
philosophy: predictable, low false-positives, no toolchain pinning. `dylint` stays
the documented upgrade path for the [non-goals](#precision-limits--non-goals) the
heuristic can't reach — exactly how the README frames it ("`dylint` for full
name-resolution precision").

## Sits on top of (sequencing)

- **#40 (Rust location, Phase 2)** introduces the first `syn` parse and a
  `Language::Rust`. This rule **reuses that foundation**; if #40 hasn't landed when
  the first slice here starts, slice 1 below bootstraps it (and #40 consumes it).
- **Inline `waiver:` mechanism** (the per-finding marker, distinct from #32's
  whole-file `exempt` list) is **not built yet** — #42 / #43 / #52 all wait on the
  same primitive. Build it **once, cross-rule**; the waiver slice here is a consumer,
  not the owner.

## Unit detection

**Scope.** Parse each `src/**/*.rs` with `syn::parse_file`. Walk every
`#[cfg(test)] mod <name>` item (conventionally `mod tests`). The module *is* "the
test's own module"; its parent (`super`) is the unit under test. Files under
`tests/` are integration crates — out of scope for the unit rule.

Two detectors run over each test module's body:

### D1 — out-of-module path calls

Flag a call expression `A::B::…::f(args)` by its **leading segment `A`**:

| Leading segment | Verdict | Why |
| --- | --- | --- |
| `super` (single) | **allow** | the unit under test |
| `self`, `Self`, `crate::<same module>` | **allow** | local |
| *bare / unqualified* (`f(…)`, `x.method()`) | **allow** | unresolvable in `syn`; presumed local/test scaffolding ([non-goal](#precision-limits--non-goals)) |
| `super::super::…` (and deeper) | **flag** | reaches past the unit into an ancestor/sibling |
| `crate::…` (cross-module) | **flag** | an explicit first-party collaborator |
| an external crate name (from `Cargo.toml`) | **flag** | an un-doubled external dep |
| effectful `std`/`core`/`alloc` path (see below) | **flag** | filesystem / clock / net / env / process |

Macros (`assert_eq!`, `vec!`, `format!`, `println!`, …) are **not** analyzed — they
are the test's assertion vocabulary, and a macro is a different AST node
(`Macro`, not `ExprCall`). A macro hiding an effectful call is a
[non-goal](#precision-limits--non-goals).

### D2 — foreign imports

Flag a `use` inside the test module whose **path root** is not `super` / `self`:

- `use super::*;` — **allow** (the one legal glob; imports the unit under test).
- `use super::Thing;` — **allow** (specific import of the unit under test).
- `use crate::other::*;` / `use external::*;` / `use crate::other::Thing;` /
  `use external::Thing;` — **flag**. The issue mandates *"no glob imports except
  `super`"*; we extend to **specific** foreign imports too, because otherwise a
  `use crate::other::Thing;` + unqualified `Thing::new()` slips past D1 (the call
  site has no path prefix). Same `super`-only carve-out applies.
- `use std::…;` — flag **only if effectful** (below); `use std::collections::HashMap;`
  is allowed.

### Effectful-`std` policy

`std` is effectful only in these subtrees — everything else (`collections`, `cmp`,
`fmt`, `iter`, `str`, `sync`, `time::Duration`, …) is pure and allowed:

| Flagged (effectful) | Allowed (pure) |
| --- | --- |
| `std::fs`, `std::net`, `std::process`, `std::env`, `std::thread`, `std::os` | `std::collections`, `std::fmt`, `std::ops`, `std::convert`, … |
| `std::time::SystemTime::now`, `std::time::Instant::now` (clock) | `std::time::Duration` |
| `std::io::{stdin,stdout,stderr}` (real handles) | `std::io::Cursor` + the `Read`/`Write`/`BufRead`/`Seek` **traits** |

The `std::io` split is deliberate: [testing.md](testing.md) makes `Cursor::new(...)`
the idiomatic in-memory unit-test tool, so `std::io` is **not** flagged wholesale —
only the real-handle entry points are. "Randomness" (README) has no general std RNG;
it's the `rand` crate, caught by the external-crate branch of D1.

## Integration detection

**Scope.** Each file under `tests/` is its own crate. Flag **doubling a first-party
item**. The clean bright-line `syn` signal:

- `#[double] use crate::…;` / `#[double] use <first-party-crate>::…;` — the
  `mockall_double` attribute swapping a real first-party item for its mock. **Flag.**
- `mock! { … }` / `#[automock]` naming a first-party path — **flag** where the path
  is syntactically resolvable; fuzzier, so the first integration slice covers the
  `#[double]` signal and leaves the rest to dylint
  ([non-goals](#precision-limits--non-goals)).

Doubling an **external** crate or effectful `std` behind a trait is the whole point —
never flagged.

### First-party vs external (the `Cargo.toml` analysis)

Parse the crate's `Cargo.toml` (reuse the existing `toml` dep) to classify D1's
leading segments and the integration double targets:

| Class | Members |
| --- | --- |
| **first-party** | the crate's own `[package].name`; `{ path = … }` deps; `[workspace]` members |
| **external** | registry/git `[dependencies]` |
| **test tooling** (not a collaborator) | `[dev-dependencies]` — `mockall`, `rstest`, `proptest`, `pretty_assertions`, … are exempt: a unit test *uses its framework for real*. |

Treating `[dev-dependencies]` as test tooling is the bright-line that keeps the
mocking machinery (`mockall`) and test frameworks legal without a hand-maintained
allowlist.

## Waiver

**Decided (#102): config-driven, not an inline marker.** The escape hatch reuses
the shipped #32 machinery — a reason-required `[[<lang>.exempt]]` entry naming the
rule — rather than a `// waiver:` comment. This is the repo's one waiver pattern
(`colocated-test` / `coverage` / `no-constant-patch` all use it), auditable in a
single config diff, and it kept the detection slices free of a bespoke
comment-parsing primitive.

```toml
[[rust.exempt]]
path = "src/widget.rs"
rules = ["no-out-of-module-call"]   # the isolation rule to lift
reason = "legacy unit reaches into store; refactor tracked in #NNN"
```

- `unit isolation` takes `--config` (default `testing-conventions.toml`); both it
  and `integration lint` filter findings through `config::resolve_exempt`.
- Each rule id (`no-out-of-module-call` / `no-out-of-module-import` /
  `no-first-party-double`, and the TS rules) is a `config::Rule` variant; matching
  is on `(rule, root-relative path)`. A reason-less or stale entry errors.
- The bright-line slices (D1/D2/integration) shipped **before** this, exactly as
  #49/#50/#51 did — the waiver is additive.

(The earlier sketch here — a per-finding `// waiver:` comment — is superseded by
#102's config-driven decision.)

## Surface & module shape

- **CLI.** Unit: `unit isolation --language rust <PATH>` (new dedicated subcommand,
  matching `unit colocated-test` / `unit coverage`). Integration: extend
  `integration lint --language rust <PATH>` (the existing home for "deterministic
  lints on integration test code"; folds into the #56 config-driven `check`).
- **Module.** New `src/isolation.rs`, parallel to `lint.rs` (Python) and
  `colocated_test.rs`. Reuse the `path:line: rule — message` output; **hoist
  `Violation`** out of `lint.rs` into a shared spot so both emitters share one shape.
- **Deps.** `syn = { version = "2", features = ["full", "visit", "parsing"] }` and
  `proc-macro2 = { version = "1", features = ["span-locations"] }` for line numbers.
  `Cargo.toml` parsing reuses the existing `toml` dependency.
- **`--language rust`** parses into a dedicated `isolation::Language` (Rust-only
  today) rather than extending the file-pairing `colocated_test::Language`, so it
  doesn't pre-empt #40's place for Rust there; the enums can unify once #40 lands.

## Precision limits / non-goals

Deliberately **not** caught by the `syn` heuristic — left to review / a future
`dylint` pass, and stated plainly (à la #19's non-goals) so nobody over-trusts green:

- A collaborator reached through an **unqualified call** with no path prefix
  (bare `foo()`, `x.method()`) — `syn` can't resolve the receiver.
- A collaborator behind a **`use … as …` rename** or a **type alias**.
- A **trait method** whose impl lives out of module (method-call syntax carries no
  resolvable path).
- An effectful call **hidden in a macro** (`println!`, a custom `macro_rules!`).
- The fuzzier integration doubles (`mock!` / `#[automock]` bodies) beyond the
  `#[double] use` signal.

D2's specific-import flagging closes the most common of these (named foreign import +
unqualified use); the rest are the documented `dylint` upgrade.

## Fixtures (per slice, red + clean — #3 guardrail)

- **Unit red:** a `src/`-shaped file whose `#[cfg(test)] mod tests` (a) calls
  `crate::other::thing()`, (b) calls `std::fs::read(...)`, (c) does `use external::*`,
  (d) does `use crate::other::*` — each a distinct flagged case.
- **Unit clean:** `use super::*;`, calls only `super::` items + assertions +
  `Cursor::new`, injects a `mockall` / hand-rolled trait double. Zero findings.
- **Integration red:** a `tests/` crate with `#[double] use crate::…;`.
- **Integration clean:** a `tests/` crate doubling only an external crate / effectful
  `std` behind a trait; first-party runs real.

## Implementation slices (red→green)

Each is its own test-first increment with CHANGELOG + MIGRATIONS + a VitePress doc.

1. **Foundation** — `syn` + `proc-macro2` deps, Rust-file collection, `Cargo.toml`
   first-party/external/dev-dep resolution, `Language::Rust`, hoisted `Violation`.
   *(Shared with / absorbed by #40.)*
2. **Unit D1** — out-of-module path calls (`crate::`, effectful `std`, external).
3. **Unit D2** — foreign imports (glob + specific; `super`-only carve-out).
4. **Integration** — flag `#[double] use <first-party>`.
5. **Waiver** — consume the shared inline `waiver:` primitive (gated on it landing).

Order: 1 → 2 → 3 → 4 → 5. Slices 2–4 are independent once 1 lands; 5 waits on the
shared waiver primitive.
