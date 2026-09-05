### `no-first-party-patch` classifies object-form chains by what they end at

**Summary**

The object form resolved `patch.object(async_mod.asyncio, "to_thread")` to
`myproject.async_mod.asyncio.to_thread` and classified it by the chain's head segment, so a
stdlib module reached through a first-party module attribute was flagged as first-party — while
the runtime-identical `patch("asyncio.to_thread")` passed. An attribute past the imported name is
now classified by the first-party module's own top-level source: an import binding names the
module it comes from, a `def` / `class` / literal assignment (or a submodule file) is
first-party, and an attribute the source leaves unnamed resolves to no target, so nothing fires.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

An integration or e2e test that patches an effectful-stdlib or third-party module through a
first-party module attribute — `patch.object(async_mod.asyncio, "to_thread", spy)`,
`patch.object(async_mod.os, "getcwd")` — now passes `no-first-party-patch`. A
`[[python.exempt]]` entry taken to work around that flag can be dropped. A first-party attribute
reached the same way still fires: `patch.object(async_mod.helper, "run")` where `async_mod.py`
holds `from . import helper`, or `patch.object(async_mod.Client, "send")` where it defines
`Client`. An attribute the module's source leaves unnamed — a dynamic assignment, a conflicting
binding, a source file the scan cannot find — is left alone.

**Verification**

Run the check from the package root over a suite that patches stdlib through a first-party
module attribute:

```sh
npx testing-conventions integration lint --language python tests/integration
```

The run prints nothing and exits 0, where it previously named the patch:

```
tests/integration/query_test.py:9: no-first-party-patch — patches a first-party target; an integration test must run first-party code for real — only third-party packages and effectful stdlib may be patched
error: 1 lint violation(s)
```
