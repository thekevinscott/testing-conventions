# TypeScript — testing

**Default to Vitest.**

Per-package config (`vite.config.ts`):

```ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    include: ['src/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'lcov'],
      thresholds: { lines: 100, functions: 100, branches: 100, statements: 100 }
    }
  }
});
```

**Integration tests** at `test/integration/` consume the *built* artifact, not source. Catches breaks (export-map drift, missing files, bad shebang on bin scripts) that source-only tests miss.

**Mocking**: use **factory injection**. Pass dependencies as constructor args; tests pass fakes:

```ts
// src/widget.ts
export function getWidget({ load, run }: Deps) {
  class Widget {
    constructor(opts: WidgetOptions) { /* ... */ }
    execute(input: Input) { /* uses load, run */ }
  }
  return Widget;
}

// src/index.ts
import { load } from './load';
import { run } from './run';
export const Widget = getWidget({ load, run });

// src/widget.test.ts
import { getWidget } from './widget';
const Widget = getWidget({ load: fakeLoad, run: fakeRun });
```

Factory injection works identically in every test runner and keeps the test plumbing visible at the call site.

**No mechanism-hygiene integration lint (by design).** Python's `integration lint` carries
three mechanism lints — `no-monkeypatch`, `no-inline-patch`, `no-environ-mutation` — that
police *how* a pytest test mocks. TypeScript has none, deliberately: factory injection has
no `monkeypatch` or in-place `os.environ` analog, and the "don't hand-roll an untyped mock"
concern is already the `untyped-mock` *unit* rule (`unit lint`). The TypeScript
`integration lint` is the first-party *direction* check alone — `no-first-party-mock` (don't
`vi.mock()` a first-party module).

**E2E attestation** — e2e tests aren't run in CI. Run them locally and attest:
`testing-conventions e2e attest 'vitest run tests/e2e'` commits a receipt naming the
commit they ran against; in CI, `e2e verify` checks it's current (re-run `attest`
when it goes stale). CI never runs the e2e suite.
