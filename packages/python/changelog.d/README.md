# Changelog fragments

One file per change, named `YYYY-MM-DD-<slug>.md` — the UTC merge date, then a kebab-case slug
(lowercase letters, digits, hyphens). This directory names the package and the kind, so the
filename carries neither.

The body opens with a bold Keep a Changelog category — `**Added**` / `**Changed**` /
`**Deprecated**` / `**Removed**` / `**Fixed**` / `**Security**` — then the entry text as it would
read in a changelog. A breaking change carries a `**BREAKING**` prefix and names its migration
fragment in `../migrations.d/`.

```markdown
**Fixed** `unit colocated-test --base` skips type-only TypeScript modules, matching the presence
rule. Editing such a module previously raised `source changed without its colocated test`.
```

A fragment is a new file, so two PRs touching this package add two different paths and never
conflict. Fragments are permanent and append-only: nothing is assembled back into
[`../CHANGELOG.md`](../CHANGELOG.md), which is a frozen archive of the entries written before this
convention.

`.github/workflows/changelog.yml` requires one fragment here per changed package on every PR.
`docs/internals/repo.md` ("CHANGELOG + MIGRATIONS") carries the full convention.
