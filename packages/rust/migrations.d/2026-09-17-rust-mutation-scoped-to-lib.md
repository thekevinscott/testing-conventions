### Rust `unit mutation` measures the library target alone

**Summary**

Rust `unit mutation` handed cargo-mutants no test-target restriction, so it fell back to its
default `cargo test`: the whole `tests/` tree built and ran once per mutant, including any e2e
tier that tier is meant to keep out of CI (`e2e verify` exists for exactly that reason). Rust
`unit coverage` already restricts to `--lib`, so the two checks measured different slices for the
same crate. The mutation run now passes `--cargo-test-arg --lib`, so only the library target's
colocated unit suite judges each mutant — parity with coverage, and with the Python and
TypeScript mutation arms, which already scope to the unit suite through the scan path.

**Required changes**

_None._ The change is inside the Rust mutation run; a `uses:` call or direct
`testing-conventions unit mutation --language rust <crate>` needs no edit.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A mutant that only an integration test or an e2e tier under `tests/` caught now survives instead
of passing: `unit mutation` reports it and the gate fails unless the survivor carries a reasoned
exemption. A crate whose `tests/` tree held a real-service e2e tier no longer executes that tier
during a mutation run.

**Verification**

Run the check over a Rust crate whose `tests/` integration suite is the only thing that catches a
mutant its library-scoped unit suite leaves unjudged:

```sh
npx testing-conventions unit mutation --language rust <crate>
```

The mutant surfaces as a survivor; the gate fails until it is either caught by a unit test or
exempted with a reason.
