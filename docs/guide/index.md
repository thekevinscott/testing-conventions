# How-to Guides

Task recipes for `testing-conventions`. Each guide is one job — reach for the one that matches what
you're trying to do. New to the tool? Run the [drop-in](../getting-started) first.

## Customize the standard

- [Configure the rules](./configure) — relax a coverage floor or exempt a file in
  `testing-conventions.toml` (the *what to enforce* surface).
- [Extend the defaults](./extending) — reuse our shared test config so a local `vitest` / `pytest`
  run is held to the same floor CI enforces.

## Understand and run the rules

- [Isolate tests](./isolation) — the unit/integration boundary and the per-language mocking rules.
- [Run mutation testing](./mutation) — verify the lines a change touches, not just execute them.

## Run it your way

- [Enforce conventions in CI](./ci) — the reusable workflow's inputs, diff-scoped checks, and
  rolling your own steps (the *where and how to run* surface).
- [Use the CLI directly](./cli) — install and run the rules yourself, without the workflow.

## See also

- [The testing model](../explanation/): *why* the standard is shaped this way.
- [Reference](../reference/): the full CLI surface and config schema.
