**Added** `workflow-lint [path]` — a GitHub Actions `run:` or `github-script` body must read as
wiring, not a program. A body is flagged when it iterates (`for` / `while` / `until` / `select`),
dispatches on `case`, runs `awk` or `sed`, or exceeds 12 command lines; blank lines, comments and a
`set -…` prologue do not count against the ceiling. Discovery follows GitHub's own rules — every
`*.yml` / `*.yaml` directly inside a `workflows/` directory, plus every `action.yml` /
`action.yaml` at any depth — so a fixture or lockfile under `.github` is never mistaken for CI. The
check reads no configuration and honors no exemptions, and a repository with no workflows is
skipped, never failed.
