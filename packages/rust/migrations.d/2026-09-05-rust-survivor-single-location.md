### A Rust survivor line carries one location

**Summary**

A Rust mutation survivor rendered its location twice —
`src/lib.rs:7: src/lib.rs:7:5: replace > with == in is_positive` — because the normalized line
prepends `file:line:` while cargo-mutants' mutant name carries its own `file:line:col:` prefix.
The embedded prefix is now stripped: the line reads
`src/lib.rs:7: replace > with == in is_positive`, one location, matching the shape of the Python
and TypeScript lines.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

Anything reading the doubled form of a Rust survivor line — the `file:line:col:` that followed
the leading `file:line:` — reads the single-location form instead.

**Verification**

Run the check over a Rust crate with a surviving mutant:

```sh
npx testing-conventions unit mutation --language rust <crate>
```

Each survivor line carries one location:

```
  src/lib.rs:7: replace > with == in is_positive
```
