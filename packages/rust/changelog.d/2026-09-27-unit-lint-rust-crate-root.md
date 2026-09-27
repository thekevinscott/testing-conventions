**Fixed** **`unit lint --language rust` over a subdirectory reads the crate manifest** (#742). The
external-crate set came from a `Cargo.toml` read at the scan path, so pointing the gate at
`<crate>/src` found no manifest, left the dependency set empty, and stopped flagging every
external-crate reach — a zero exit over a tree with violations. This crate measured 34 at
`packages/rust` and 6 at `packages/rust/src`. The manifest is now resolved from the nearest
ancestor holding `Cargo.toml`, matching what `integration lint` already did, and both scan paths
report the same 34.

Exempt paths moved with it: `[[rust.exempt]].path` is now relative to that crate root rather than
to the scan path, so one entry resolves from either. A tree that scanned the crate root and named
its paths `src/…` is unaffected. A tree that scanned `<crate>/src` and named paths relative to it
must add the `src/` prefix — and was silently passing on external-crate reaches before this, so
expect new findings there.
