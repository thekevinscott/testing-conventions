---
description: Why test files never ship in the built artifact — the cost colocation imposes, and how the packaging check settles it per registry.
---

# Packaging

Colocated unit tests are the standard's core structural move — and they have one cost: the tests
live in `src/`, exactly where a naive build scoops them up. The `packaging` check settles that
cost: it inspects the **built artifact** — the thing you actually publish — and fails if any test
file shipped.

## Why the built artifact, not the source tree

Every other check reads the source tree; this one reads the wheel, tarball, or crate. That's the
only honest place to check: whether tests ship is a property of the *build configuration* (the
wheel's include list, the npm `files` field, the Cargo `exclude`), and the artifact is where a
build-config regression becomes visible. A source-tree check would pass forever while a changed
manifest quietly started shipping `*_test.py` to every consumer.

## What it enforces

The check unpacks each distribution and scans for the language's test pattern:

- **Python** — no `*_test.py` in the wheel (`.whl`) or sdist (`.tar.gz`).
- **TypeScript** — no `*.test.*` in the `npm pack` tarball (`.tgz`).
- **Rust** — no crate-root `tests/` directory in the `.crate`. Inline `#[cfg(test)]` units compile
  out of the consumer artifact for free; only the integration `tests/` needs a Cargo `exclude`.

In the [workflow](../reference/workflow) the check is **locate-or-skip**: it runs over a
conventional `dist/` found in the checkout, or over a built artifact you upload and name via the
`packaging_artifact` input — and is skipped, never failed, when neither exists, so the drop-in is
safe on a repository that hasn't built anything yet. A named artifact holding no recognized
distribution is a misconfigured upload, and that fails.
