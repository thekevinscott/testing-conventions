### A Rust unit test may read the filesystem

**Summary**

`std::fs` and `std::env::temp_dir` are no longer effectful `std` for the purposes of
`no-out-of-module-call` and `no-out-of-module-import`. A unit test may build the tree its unit
walks. Every other effectful subtree — `net`, `process`, `thread`, `os`, the clock, real `io`
handles, and the rest of `std::env` — is unchanged.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

A unit test that calls `std::fs::*` or imports `use std::fs` no longer fails `unit lint
--language rust`. If such a file carries a `[[rust.exempt]]` entry for
`no-out-of-module-call` or `no-out-of-module-import`, and the filesystem was the only reason for
it, that entry is now stale — and a stale exempt entry is an error, not a silent pass. Delete it.

`std::env::var` and its siblings still fail. Only `std::env::temp_dir` is allowed, because it
names a writable directory rather than reading ambient state.

**Verification**

Run the check from the package root over a unit test that reads the filesystem:

```sh
npx testing-conventions unit lint --language rust src
```

A test that only touches `std::fs` and `std::env::temp_dir` exits 0. One that reads an
environment variable still names the constraint and exits 1:

```
src/config.rs:31: no-out-of-module-call — unit test calls `std::env::var` out of its own module (effectful std); inject a trait double — only `super::` is in-module
```
