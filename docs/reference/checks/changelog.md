---
description: "The changelog check — a pull request that changes a package's public surface adds a changelog fragment and, where the repository keeps one, a migration fragment. Covers fragment discovery, the two layouts, the filename convention, and the bypass."
---

# `changelog`

A pull request that changes a package's public surface adds a **fragment** recording the change.
This page is the complete record of the check: why fragments replace a shared file, how the
repository's layout is discovered, what the check enforces, and its configuration surface.

## Why this check exists

A single `CHANGELOG.md` that every pull request appends to at the same anchor conflicts by
construction. Two branches touching one package both edit the same line range, so the second to
merge resolves a conflict it had no part in creating — a cost that scales with how many people
are shipping at once, which is exactly when it hurts most.

A fragment is a **new file**, so two pull requests against one package add two different paths and
the merge is clean. The release step collects the fragments into the published notes; the working
tree holds the record in between.

## What it enforces {#enforces}

For every package whose public surface the pull request changed:

- One **added** file under that package's `changelog.d/`.
- One **added** file under that package's `migrations.d/`, where the repository keeps migration
  fragments.

A file that was modified doesn't satisfy the check — the gate reads the diff filtered to
additions, because a fragment satisfies it only when the pull request writes a new one.

### The filename convention

`YYYY-MM-DD-<slug>.md` — the UTC merge date, then a kebab-case slug of lowercase letters, digits,
and hyphens. A fragment whose name doesn't match is reported against its own path, so the
annotation lands on the offending file.

The date prefix sorts the directory chronologically, which is what lets the release step read the
fragments in order. The slug is free text: a repository pooling fragments from several packages in
one directory writes the package into the slug (`2026-09-21-parser-drop-legacy-flag.md`) and the
name parses unchanged.

### What a package may change without owing a fragment

A path is exempt when it isn't public surface:

- The frozen `CHANGELOG.md` / `MIGRATIONS.md` archive — the record a fragment replaces.
- The fragment directories themselves.
- `e2e-attestations/` — CI freshness receipts, written by automation.
- Test files, in the shapes
  [`colocated-test`](./colocated-test) already derives for the package's language.

## The two layouts

The check derives its shape from **where the fragment directories are**, so the layout is a fact
about the repository rather than something you declare.

- **Per package.** `packages/<pkg>/changelog.d/` — the fragment's owning package is its parent
  directory, and the check reports one finding per package that owes one.
- **Pooled at the repository root.** A single `changelog.d/` outside any package — the check runs
  repository-scoped: a pull request that changed public surface anywhere adds one fragment.

## When it runs

When the repository holds at least one fragment directory; **skipped, never failed** otherwise, so
the drop-in is safe on a repository that doesn't keep a changelog this way.

Migration fragments are enforced when the repository keeps a `migrations.d/`. A repository that
wants changelog entries alone keeps `changelog.d/` and the migrations arm stays quiet.

Run it from the CLI, naming the pull request's base:

```sh
npx testing-conventions changelog --base "$BASE"
```

The reusable workflow runs it as its own job in a following release, where the
[`gates` input](/reference/workflow#inputs) will name it `changelog`.

## The bypass

A `skip-changelog: <reason>` line on any commit in the pull request bypasses the check, for a
change that is genuinely an internal refactor. The line is matched anywhere in any commit body
rather than parsed as a git trailer, so it works from a commit whose body already carries other
text.

The reason is required, and it is read by whoever reviews the pull request — the check records
that a human made the call, it doesn't judge the reason.

## Configuration

The check reads no configuration. The fragment directories are discovered, the filename
convention is fixed, and whether migrations are enforced follows from whether the repository
keeps that directory.

The check honors no exemption rules — a package's public surface either got a fragment or it
didn't.
