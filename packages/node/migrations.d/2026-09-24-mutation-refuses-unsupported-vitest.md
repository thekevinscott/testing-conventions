### The mutation gate refuses an unsupported vitest

**Summary**

`unit mutation` reported every TypeScript mutant as `survived` on vitest 5, whatever the suite
asserted. Stryker's vitest runner runs no tests per mutant from that major on, and scores "no
tests ran" as a surviving mutant rather than an error, so the gate was a false report rather than
a failing measurement. The adapter now reads the vitest the project resolves and fails outright
when it cannot judge mutants against it.

**Required changes**

Pin the project to a vitest the gate can measure:

```jsonc
// package.json
"devDependencies": {
  "vitest": "^4",
  "@vitest/coverage-v8": "^4"
}
```

A project that cannot move off vitest 5 leaves `mutation` out of the reusable workflow's `gates`
input. The allowlist is authoritative, so the job is skipped:

```yaml
with:
  gates: '["colocated-test", "unit-lint", "unit-coverage"]'
```

**Deprecations removed**

_None._

**Behavior changes without code changes**

A vitest 5 project that was **green** because `mutation` exemptions lifted the false survivors now
fails, naming the vitest it found. Those exemptions were suppressing a bug, not a real coverage
gap — after pinning to vitest 4, drop them and re-run: mutants the suite actually kills report as
killed. An exemption cannot lift this failure, because it applies to results the adapter returns
and the adapter now returns none.

A vitest 5 project that was **red** with unexplained survivors stays red, with a message that
names the cause instead of pointing at source lines whose tests already pass.

Nothing changes on vitest 4 or below.

**Verification**

On vitest 5, the gate fails with the cause rather than a mutant list:

```
the mutation gate cannot judge mutants on vitest 5.0.1: Stryker's vitest runner runs no tests
per mutant on vitest 5 and above ...
```

After pinning to `^4`, the same suite reports its mutants as killed.
