### The pooled fragment layout is scoped to packages

**Summary**

Under the pooled layout — one `changelog.d/` outside every package, as a repository keeping
`docs/changelog.d/` has — the check read the whole tree as public surface and accepted any
well-formed fragment name. A root `README.md` edit owed fragments, and a fragment written for
package `bar` paid for a change to package `foo`.

Pooled mode now works the way the per-package layout does. Public surface is scoped to the
repository's package roots, and a pooled fragment pays for the package its name identifies.

**Required changes**

A pooled repository names the package in each fragment it adds, for both kinds:
`docs/changelog.d/2026-10-06-parser-drop-the-legacy-flag.md`, and under a `breaking:` line
`docs/migrations.d/2026-10-06-parser-drop-the-legacy-flag.md`. A fragment naming another package
leaves this one unpaid:

```
::error::packages/parser changed public surface without adding a changelog fragment. Add
changelog.d/YYYY-MM-DD-parser-<slug>.md, …
```

Library consumers of the Rust crate: `changelog::Layout::Pooled` carries the repository's package
roots, as `Layout::PerPackage` carries its container directories —
`Layout::Pooled(vec!["packages/parser".to_string()])`, and `Layout::Pooled(Vec::new())` for a
repository with no discoverable package root. `changelog::discover_layout` fills it in.
`changelog::names_package` is the new name test.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A pooled repository stops owing fragments for paths outside every package: a root `README.md`, a
`docs/` page, repository tooling. It starts owing a correctly named fragment per changed package,
where one fragment under any name used to satisfy the whole pull request.

A **package root** is a directory inside a container directory holding a `package.json`,
`pyproject.toml` or `Cargo.toml`. A repository where none is discoverable keeps the previous
whole-tree behaviour, so a single-package repository with a pooled `changelog.d/` is unaffected.

The per-package layout is untouched: its fragment's owning package is still its parent directory,
and its filename still carries a free-text slug.

**Verification**

In a repository keeping `docs/changelog.d/` and a package at `packages/parser`:

```sh
printf 'edited\n' >> README.md && git commit -qam 'docs: edit the readme'
npx testing-conventions changelog --base "$(git merge-base origin/main HEAD)"
# Every scope that changed public surface added its fragments.
```

Change `packages/parser/src/` instead and add
`docs/changelog.d/$(date -u +%F)-lexer-something.md`; it exits 1, naming `packages/parser` and the
fragment name that would pay.
