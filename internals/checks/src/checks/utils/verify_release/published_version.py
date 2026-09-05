"""The npm version a release published, read from its tags."""
from __future__ import annotations

NPM_TAG_PREFIX = "testing-conventions-npm-v"


def published_version(tags: list[str]) -> str:
    """The highest npm version among `testing-conventions-npm-v*` tags, as a bare `X.Y.Z` —
    numerically sorted, since a lexical sort puts `v0.0.9` above `v0.0.67`. Raises when no npm
    tag is present: the release published no binary to pin to, so promotion must not proceed."""
    versions = [tag[len(NPM_TAG_PREFIX):] for tag in tags if tag.startswith(NPM_TAG_PREFIX)]
    if not versions:
        raise ValueError(
            f"no {NPM_TAG_PREFIX}* tag reachable from the release commit — no published npm "
            "version to pin the verification to; refusing to promote (#357)"
        )
    return max(versions, key=version_key)


def version_key(version: str) -> tuple[int, ...]:
    return tuple(int(part) for part in version.split("."))
