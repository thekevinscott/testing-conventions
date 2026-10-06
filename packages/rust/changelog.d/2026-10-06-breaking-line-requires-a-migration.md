**BREAKING** `changelog` asks for a migrations fragment when a commit in the pull request carries
a `breaking: <summary>` line, in place of asking for one on every public-surface change the moment
the repository keeps a `migrations.d/`. A repository recording migration notes for the breaking
subset of its history no longer writes an empty fragment per pull request. A migrations fragment
added without the line stays allowed. See `migrations.d/2026-10-06-breaking-line-requires-a-migration.md`.
