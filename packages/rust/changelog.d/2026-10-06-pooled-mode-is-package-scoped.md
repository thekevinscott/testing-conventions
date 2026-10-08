**BREAKING** `changelog` scopes the pooled fragment layout to packages. Public surface is what
sits under a discovered package root, so a root `README.md`, a `docs/` page, or repository tooling
outside every package owes no fragment; and a pooled fragment pays only for the package its
`YYYY-MM-DD-<pkg>-<slug>.md` name identifies, for both fragment kinds. A repository with no
discoverable package root keeps the whole-tree behaviour. The per-package layout is unchanged. See
`migrations.d/2026-10-06-pooled-mode-is-package-scoped.md`.
