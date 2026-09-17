**Fixed** `e2e verify`'s second question now reads only the acting branch's own receipt
(`e2e-attestations/<branch_slug>.json`), not any file under `e2e-attestations/`. Updating a
receipt left behind by a different, already-merged branch no longer satisfies the nudge. A new
`--branch <name>` flag supplies the acting branch on a detached-HEAD checkout, which the reusable
workflow's fork-safe checkout produces; absent both a resolvable branch and `--branch`, `verify`
falls back to the prior whole-directory question. See
`../migrations.d/2026-09-17-e2e-verify-scopes-to-own-receipt.md`.
