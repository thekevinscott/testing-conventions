# Migrations

Upgrade notes for breaking changes. New entries go under `## Unreleased`.
On release, the section is renamed to `## v<OLD> → v<NEW>`.

Each entry has five sections, in order:

1. **Summary** — one paragraph: what changed and why.
2. **Required changes** — before/after for public API. "None" if purely additive.
3. **Deprecations removed** — anything previously warned about that's now gone.
4. **Behavior changes without code changes** — same API, different runtime behavior.
5. **Verification** — commands that confirm the upgrade worked, with expected output.

## Unreleased

### Summary

Adds a `vitestConfig` export to the package root so consumers extend the shared
coverage floor instead of copy-pasting it. Purely additive: the `bin` entry and
its CLI behavior are unchanged, and `vitest` is added as an *optional* peer
dependency, so CLI-only consumers see no new install requirement.

### Required changes

None.

### Deprecations removed

None.

### Behavior changes without code changes

None.

### Verification

None.
