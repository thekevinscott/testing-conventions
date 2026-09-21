### The changelog check ships in the CLI

**Summary**

The changelog-fragment gate moves out of this repository's private `internals/checks` package and
into the published binary as `testing-conventions changelog --base <base>`. Every consumer gets
the rule from the same release as the other checks, rather than reimplementing it.

The check derives its whole shape from the repository. Fragment directories are discovered by
looking for them: under `packages/<pkg>/` it scopes findings per package, and a single
`changelog.d/` outside any package scopes them to the pull request. Migration fragments are
enforced where a `migrations.d/` exists.

**Required changes**

_None._ The command is additive; existing subcommands are unchanged.

**Deprecations removed**

_None._

**Behavior changes without code changes**

_None._ The reusable workflow does not run the check yet — the job wiring lands in the following
release, once the published binary understands the subcommand.

**Verification**

Run `npx testing-conventions changelog --base "$(git merge-base origin/main HEAD)"` at a
repository root. A repository with no `changelog.d/` exits 0 with a skip message.
