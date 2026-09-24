### Rust `unit coverage` and `unit mutation` measure binary targets too

**Summary**

`colocated-test` requires an inline `#[cfg(test)]` module in every Rust source file that holds
testable code. `src/main.rs` is such a file, so a binary crate had to carry tests there — but both
`unit coverage` and `unit mutation` restricted the suite to `--lib`, which never builds or runs a
binary target's test modules. The gate demanded tests that the runs then ignored: they could not
fail, could not count toward the coverage floor, and could not kill a mutant. Both runs now pass
`--bins` alongside `--lib`, so a binary target's colocated suite executes.

`--bins` and not `--all-targets`: the latter also pulls in the `tests/` integration tier, which
`--lib` exists to exclude and which
`../migrations.d/2026-09-17-rust-mutation-scoped-to-lib.md` deliberately removed.

**Required changes**

A crate whose binary source holds logic now has that logic measured. Either write the colocated
tests it was always nominally required to have, or move the logic into the library and reduce the
binary root to a declaration. Two shapes qualify.

A re-export carries no function at all, so `colocated-test` asks for no test module and the file
emits no code regions — it is absent from the coverage report entirely:

```rust
// src/main.rs
pub use mycrate::entrypoint::main;
```

Or delegate in a single call. `colocated-test` now reads a `fn main` whose body is one argument-free
call and nothing else as a declaration too:

```rust
// src/main.rs
#[cfg(not(test))]
fn main() -> std::process::ExitCode {
    mycrate::entrypoint::main()
}
```

`#[cfg(not(test))]` is required on that shape. Without it the function is instrumented and reads 0%,
because `cargo test` replaces a binary's `main` with the harness's own and no unit test can execute
it. The re-export needs no attribute.

Either way the re-exported or called `main` lives in the library, where the logic it wires is under
test. Rust accepts a re-export as the binary's entry point as long as the item is a zero-argument
function returning a `Termination` type.

**Deprecations removed**

_None._

**Behavior changes without code changes**

- A binary target's inline `#[cfg(test)]` tests now run as part of `unit coverage` and
  `unit mutation`. A failing one fails the gate where it was silently skipped before.
- Uncovered lines in a binary source now count against the coverage floor.
- A surviving mutant in a binary source now fails `unit mutation` unless exempted with a reason.
- A binary root holding only a logic-free `fn main` stops being a `colocated-test` subject. A
  `colocated-test` exemption covering that file is now redundant and worth deleting. Nothing flags
  it: stale-entry detection fires on a path that matches no file, and the file still exists.

One line stays outside the unit tier by construction: `cargo test` replaces a binary's `main` with
the harness's own, so `fn main` is never executed by a unit test, and calling it from one would run
the real CLI over the harness's argv. Keep `main` to the process-boundary argv read with no
decision in it — mark it `#[cfg(not(test))]` so it is excluded from the report rather than sitting
permanently uncovered — and put every decision in a function a test can call.

**Verification**

Add a failing inline test to a binary source and run either check over the crate:

```sh
npx testing-conventions unit coverage --language rust <crate>
```

The test now runs, and the file appears in the coverage report. Before this change the run passed
without ever compiling it.
