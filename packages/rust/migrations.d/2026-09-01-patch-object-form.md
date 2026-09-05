### `no-first-party-patch` and `no-constant-patch` cover `patch.object` / `patch.dict`

**Summary**

Both rules previously read only a string-literal `patch("...")` target, so the object forms of
the exact patches they forbid passed clean. The first argument of `patch.object(...)` /
`patch.dict(...)` is now resolved through the file's imports to a dotted target and held to the
same predicates.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

An integration or e2e test that patches a first-party target through the object form — after
`from myproject import ledger`, `patch.object(ledger, "record")` or
`patch.dict(config.registry, {...})` — now fails `integration lint` with `no-first-party-patch`.
One that patches an UPPER_CASE config attribute (`patch.object(cfg, "CACHE_DIR", tmp_path)`) now
fails with `no-constant-patch`. Rework the test to mock at the system boundary (or inject the
config), or take a reason-required `[[python.exempt]]` entry. A base bound by no import
(`patch.object(get_mod(), "x")`) still resolves to no module and is left alone.

**Verification**

Run the check from the package root over a suite that patches first-party code via the object
form:

```sh
npx testing-conventions integration lint --language python tests/integration
```

The CLI runs on node 24 or newer — npm resolves a bare name to the newest release the running node
satisfies.

The run names the patch and exits 1:

```
tests/integration/charge_test.py:12: no-first-party-patch — patches a first-party target; an integration test must run first-party code for real — only third-party packages and effectful stdlib may be patched
error: 1 lint violation(s)
```
