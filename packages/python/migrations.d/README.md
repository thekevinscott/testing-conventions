# Migration fragments

One file per change, named `YYYY-MM-DD-<slug>.md` — the UTC merge date, then a kebab-case slug
(lowercase letters, digits, hyphens). This directory names the package and the kind, so the
filename carries neither.

Each fragment is a complete entry: a `### <title>` heading and five sections, in order.

```markdown
### One subject predicate for presence and co-change

**Summary**

...

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

...

**Verification**

...
```

Keep every heading; write `_None._` under one that does not apply.

A fragment is a new file, so two PRs touching this package add two different paths and never
conflict. Fragments are permanent and append-only: nothing is assembled back into
[`../MIGRATIONS.md`](../MIGRATIONS.md), which is a frozen archive of the entries written before
this convention.

`.github/workflows/changelog.yml` requires one fragment here per changed package on every PR.
`docs/internals/repo.md` ("CHANGELOG + MIGRATIONS") carries the full convention.
