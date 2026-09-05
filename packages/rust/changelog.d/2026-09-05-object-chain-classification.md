**Fixed** `no-first-party-patch` classifies an object-form target by what the resolved chain ends
at, not where it starts. `patch.object(async_mod.asyncio, "to_thread")` patches the stdlib
`asyncio` module, so reaching it through a first-party module attribute no longer fires. The
attribute is read from the first-party module's own top-level source: an import binding
classifies it by the module it names, a `def` / `class` / literal assignment (or a submodule
file) is first-party, and an attribute the source leaves unnamed resolves to no target — nothing
fires. See
[`../migrations.d/2026-09-05-object-chain-classification.md`](../migrations.d/2026-09-05-object-chain-classification.md).
