# Exempt a file you can't (or shouldn't) test

A blocking gate is only honest if it has an escape hatch for the files that genuinely
shouldn't be tested — a launcher shim, a pure re-export barrel, generated code. Otherwise the
checker forces pointless tests or fights your existing conventions, and someone disables it.

`testing-conventions` gives you two escape hatches, and **neither is a silent ignore**:
structural exemptions are deterministic (no configuration), and waivers are reason-required
and visible in the file's own diff.

## Let a barrel through (automatic)

A pure re-export **barrel** — a TypeScript file whose only statements are `export … from "…"`
— carries no logic of its own, so there's nothing to unit-test. It's exempt from
`unit location` automatically, matched by **shape, not name**:

```ts
// src/index.ts — no colocated index.test.ts needed
export * from './widget';
export { Button } from './button';
export type { ButtonProps } from './button';
```

This is the TypeScript analog of Python's `__init__.py`. The match is purely structural — the
moment the file gains a local declaration (`export const`, `export function`, an `import`), it
becomes a subject again and needs its colocated test.

## Waive a file explicitly (reason-required)

For a deliberate omission the tool can't infer, drop an in-file marker — a comment, anywhere
in the file:

```
testing-conventions:waiver(<scope>): <reason>
```

- **`<scope>`** is `location` (skip the colocated-test requirement), `coverage` (omit from the
  coverage denominator), or `all` (both).
- **`<reason>`** is everything to the end of the line, and it's **required**.

### A launcher shim with no unit test

```ts
// src/cli.ts
// testing-conventions:waiver(location): thin CLI launcher; logic lives in run(), tested in run.test.ts
export const main = () => process.exit(cli(process.argv));
```

`unit location` now skips `cli.ts` instead of reporting it as an orphan.

### Generated code you don't want in the coverage number

```python
# src/pb/messages.py
# testing-conventions:waiver(coverage): generated protobuf stubs, not hand-authored
```

`unit coverage` omits `messages.py` from the denominator, so it can't drag the total below
your floor.

## Why this beats an ignore-list

A waiver lives **at** the omission, carries a **reason**, and shows up in code review and the
file's diff — auditable by construction. And the marker token `testing-conventions:waiver` is
**reserved**: if it isn't a valid `(scope): reason`, the check **errors** instead of passing,
so a typo (or a reason-less omission) can never quietly disable the gate.

```sh
# A reason-less waiver fails loudly:
$ testing-conventions unit location --language typescript src/
error: checking waivers in `src/cli.ts`: waiver missing a reason: …
```

## See also

- [Reference — Exemptions & waivers](../reference/#exemptions-waivers) — the exact grammar and
  the full table of structural exemptions.
