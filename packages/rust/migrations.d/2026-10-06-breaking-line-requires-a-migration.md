### A `breaking:` line is what requires a migrations fragment

**Summary**

The `changelog` check asked for a migrations fragment on every public-surface change as soon as the
repository kept a `migrations.d/` directory. That directory records the breaking subset of a
repository's history, so the rule made every repository keeping migration notes write an empty
fragment per pull request.

The pull request now states the fact: a `breaking: <summary>` line on any commit in
`<base>..HEAD` requires the migrations fragment, and without one a changelog fragment alone pays.
The signal mirrors `skip-changelog:` — matched anywhere in any commit body, case-insensitively —
so the check still reads no configuration. A migrations fragment added without the line is
allowed; the line makes one required, never forbidden.

**Required changes**

A pull request that breaks a consumer adds the line to any of its commits:

```
feat!: drop the legacy flag

breaking: the --legacy flag is gone; pass --mode legacy instead
```

Before, the migrations fragment was owed by every pull request touching public surface. After, a
pull request that owes one and does not add it reads:

```
::error::packages/parser marked a breaking change without adding a migrations fragment.
```

Library consumers of the Rust crate: `changelog::migrations_enforced` is removed —
`changelog::has_breaking_line(&bodies)` answers the same question from the pull request's commit
bodies instead of from the repository's directories.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A repository keeping a `migrations.d/` passes with a changelog fragment alone where it previously
failed. A repository that wants the old every-change enforcement puts a `breaking:` line on the
commits that break something, which is the only case the fragment is read for.

**Verification**

In a repository keeping both fragment directories, with one package's source changed and a
changelog fragment added:

```sh
npx testing-conventions changelog --base "$(git merge-base origin/main HEAD)"
# Every scope that changed public surface added its fragments.
```

Amend any commit to carry `breaking: <summary>` and run it again; it exits 1, naming the
migrations fragment the package owes.
