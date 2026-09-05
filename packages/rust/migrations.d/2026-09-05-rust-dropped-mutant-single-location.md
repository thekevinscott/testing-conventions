### A dropped-mutant site carries one location

**Summary**

The Rust zero-mutant proof — the hard error raised when cargo-mutants reports no mutants while the
crate's own mutant list puts one on a changed line — rendered each dropped site's location twice:
`src/lib.rs:7: src/lib.rs:7:7: replace > with == in is_positive`. The site prepends `file:line:`
while cargo-mutants' mutant name carries its own `file:line:col:` prefix. The embedded prefix is
now stripped, so the site reads `src/lib.rs:7: replace > with == in is_positive` — the same shape
the survivor line already carries.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

Anything reading the doubled form of a dropped-mutant site — the `file:line:col:` that followed the
leading `file:line:` — reads the single-location form instead. A mutant name that carries no
embedded prefix renders unchanged, with its location.

**Verification**

Run the diff-scoped check over a Rust crate whose changed lines hold mutant sites:

```sh
npx testing-conventions unit mutation --language rust --base <sha> <crate>
```

When the engine's changed-line filter drops real mutants, each site in the error carries one
location:

```
  src/lib.rs:7: replace > with == in is_positive
```
