# Extend the defaults

Every rule ships a strict default — a 100% coverage floor with branch coverage on — so a new
library is held to the full standard with no config at all. This guide covers the two ways to
build on those defaults: adjust them through config, and reuse our shared test configs in your own
local setup.

## Adjust a floor or exempt a file

A `testing-conventions.toml` at your repo root refines any rule. Lower a coverage floor, or declare
a reason-required exemption — anything you omit keeps its default:

```toml
# Relax the Python floor below the strict default 100:
[python]
coverage = { branch = true, fail_under = 90 }

# Exempt a launcher shim; explicit, and a reason is required:
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test", "coverage"]
reason = "thin launcher; logic in run(), tested in run_test.py"
```

A partial table is a partial override: set just the one field you want to move and the rest of the
table keeps its default. See [Configuration](../reference/#configuration) for every key,
[Defaults](../reference/defaults) for the baseline, and [Exempt a file](./exemptions) for the
exemption rules in full.

## Reuse our shared test config

The coverage floor the rules enforce is also published as a config you can extend, so a local test
run is held to the same standard CI checks — a shortfall surfaces before you push, not after.

### TypeScript: extend `vitestConfig`

The npm package exports a ready-made vitest config from its root. Extend it with `mergeConfig`
rather than copy it (and drift from it):

```ts
// vite.config.ts
import { defineConfig, mergeConfig } from 'vitest/config';
import { vitestConfig } from 'testing-conventions';

export default mergeConfig(
  vitestConfig,
  defineConfig({
    // project-specific overrides only
  }),
);
```

`vitestConfig` carries the v8 provider, the `src/**` coverage scope (declaration files excluded),
and the `100/100/100/100` thresholds — the same TypeScript default the rule applies. The numbers
are one recommendation expressed on a second surface, so your local run and CI never disagree.

`vitest` is an optional peer dependency you already have, and the import resolves to the library
entry (separate from the CLI), so it runs no shim.
