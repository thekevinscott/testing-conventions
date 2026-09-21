**Added** `changelog --base <base>` — a pull request that changes a package's public surface adds
a fragment recording it. The fragment layout (per-package or pooled at the repository root),
whether migration fragments are enforced, and the exempt test shapes are all derived from the
repository, so the check reads no configuration. A repository that keeps no fragment directories
is skipped, never failed.
