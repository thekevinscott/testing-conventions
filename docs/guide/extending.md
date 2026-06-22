---
description: Reuse the shared test config (vitestConfig / the pytest plugin) so a local test run is held to the same floor CI enforces.
---

# Extend the defaults

This is the **local test-runner** surface: the coverage floor the rules enforce is also published as
a config you can extend, so a local test run is held to the same standard CI checks — a shortfall
surfaces before you push, not after. (To change *what* CI enforces, that's the
[config file](./configure); this only mirrors it into your own `vitest` / `pytest` setup.)

## TypeScript: extend `vitestConfig`

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
and the `100/100/100/100` thresholds — the same TypeScript default the rule applies. The numbers are
one recommendation expressed on a second surface, so your local run and CI never disagree.

`vitest` is an optional peer dependency you already have, and the import resolves to the library
entry (separate from the CLI), so it runs no shim.

## See also

- [Configure the rules](./configure): change *what* CI enforces, via `testing-conventions.toml`.
- [Reference — Configuration](../reference/#configuration): the coverage keys these mirror.
