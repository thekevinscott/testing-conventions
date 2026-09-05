"""The pass/fail messages for the version-pinned suite verification."""
from __future__ import annotations


def verification_error(sha: str, failed: list[str]) -> str:
    """The fail-closed message when one or more version-pinned suites did not pass."""
    return (
        "the version-pinned verification failed for " + ", ".join(failed)
        + f" at {sha}; refusing to promote (#357)"
    )


def verification_ok(sha: str, workflows) -> str:
    return f"the version-pinned verification passed for {', '.join(workflows)} at {sha}"
