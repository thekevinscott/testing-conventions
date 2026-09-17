**Changed** `unit mutation` now shares its subject predicate with `unit colocated-test`: a
declaration-only file — a re-export barrel, a constants module, a type-only file, and in Rust a
file with no `fn` anywhere — is never a mutation subject. Any mutant the engine finds there is
dropped before judging, so a `TIMEOUT = 30 * 60`-style constant produces no survivor to explain.
A diff touching only declaration-only modules reports `unit mutation: no mutatable changed lines
— engine not run` for TypeScript (filtered out before Stryker runs) or `unit mutation: the engine
found no mutants to test` for Python and Rust (the engine ran; its mutants were dropped). Rust's
`is_subject` arm, previously a stub matched by no caller, now backs this. See
`../migrations.d/2026-09-17-mutation-declaration-only-modules.md`.
