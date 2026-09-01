"""The changelog-gate decisions — repo-only (#496).

Pure predicates behind the `tc-checks changelog-gate` command. A PR that changes public source
under `packages/<pkg>/` must add one fragment to each of that package's fragment directories:
`packages/<pkg>/changelog.d/` and `packages/<pkg>/migrations.d/`, named `YYYY-MM-DD-<slug>.md`.

The fragment layout exists so concurrent PRs stop conflicting. A shared file that every PR appends
to at the same anchor collides by construction; a fragment is a new file, so two PRs against one
package add two different paths.

The family lives in one module because it is one decision surface — every predicate answers some
part of "does this diff satisfy the gate" over the same path vocabulary.
"""
from __future__ import annotations

import re
from collections.abc import Iterable
from typing import Optional

ROOT = "packages"

# The two fragment kinds, in the order the gate reports them missing.
KINDS = ("changelog", "migrations")

# Scanned over raw commit bodies rather than git's trailer parser, so the bypass works from any
# line of any commit — a blank line before it does not split it off as git trailers require.
_SKIP_LINE = re.compile(r"(?im)^skip-changelog:")

# Captures (<package dir>, <kind>, <filename>). The trailing `[^/]+` forbids nesting, so a stray
# subdirectory under a fragment dir is not a fragment.
_FRAGMENT = re.compile(rf"({ROOT}/[^/]+)/({'|'.join(KINDS)})\.d/([^/]+)")

# An ISO date, a kebab-case slug, and `.md`; the date is the UTC merge date.
_FRAGMENT_NAME = re.compile(r"\d{4}-\d{2}-\d{2}-[a-z0-9-]+\.md")


def has_skip_line(commit_messages: str) -> bool:
    """True when any commit body has a line opening with `skip-changelog:`."""
    return _SKIP_LINE.search(commit_messages) is not None


def changed_packages(changed: Iterable[str]) -> list[str]:
    """The package directories (`packages/<name>`) the paths touch, unique and sorted."""
    return sorted(
        {
            f"{parts[0]}/{parts[1]}"
            for path in changed
            if len(parts := path.split("/")) > 2 and parts[0] == ROOT
        }
    )


def is_exempt(path: str, pkg: str) -> bool:
    """True when `path` is not source: a frozen stub, a fragment, a receipt, or a test.

    `CHANGELOG.md` / `MIGRATIONS.md` are the frozen pre-fragment archive and the `changelog.d/` /
    `migrations.d/` folders are the entries themselves, so neither can be the change that demands
    an entry. `e2e-attestations/` holds CI freshness receipts written by automation. The test
    shapes are this repo's three: a `tests/` suite tier, a colocated Python `*_test.py`, and a
    colocated TypeScript `*.test.*` / `*.spec.*`.
    """
    p = re.escape(pkg)
    return bool(
        re.fullmatch(rf"{p}/(CHANGELOG|MIGRATIONS)\.md", path)
        or re.match(rf"{p}/({'|'.join(KINDS)})\.d/", path)
        or re.match(rf"{p}/e2e-attestations/", path)
        or re.match(rf"{p}/(tests?|__tests__)/", path)
        or re.fullmatch(rf"{p}/.*_test\.py", path)
        or re.fullmatch(rf"{p}/.*\.(test|spec)\.(ts|tsx|js|mjs|cjs|py|rs)", path)
    )


def code_touched(changed: Iterable[str], pkg: str) -> bool:
    """True when `pkg` has source changes — anything under it that is not exempt."""
    return any(
        path.startswith(f"{pkg}/") and not is_exempt(path, pkg) for path in changed
    )


def fragment(path: str) -> Optional[tuple[str, str, str]]:
    """`(package dir, kind, filename)` when `path` is a fragment, else `None`."""
    match = _FRAGMENT.fullmatch(path)
    return (match.group(1), match.group(2), match.group(3)) if match is not None else None


def malformed_fragments(changed: Iterable[str]) -> list[str]:
    """Touched fragment paths whose filenames break the naming convention.

    Each fragment directory carries a `README.md` describing the convention, which is not an entry.
    """
    return [
        path
        for path in changed
        if (frag := fragment(path)) is not None
        and frag[2] != "README.md"
        and _FRAGMENT_NAME.fullmatch(frag[2]) is None
    ]


def missing_kinds(added: Iterable[str], pkg: str) -> list[str]:
    """The fragment kinds `pkg` still owes, in `KINDS` order.

    Only *added* paths count: editing an existing fragment is not a new entry.
    """
    present = {
        frag[1]
        for path in added
        if (frag := fragment(path)) is not None
        and frag[0] == pkg
        and _FRAGMENT_NAME.fullmatch(frag[2]) is not None
    }
    return [kind for kind in KINDS if kind not in present]
