"""The pass/fail messages the verify-release commands report."""
from __future__ import annotations


def layout_error(sha: str, absent: list[str]) -> str:
    """The fail-closed message when the release archive is missing `detect`-action fetch targets."""
    return (
        f"the release archive of {sha} is missing " + ", ".join(absent)
        + " — a consumer's remote `detect` action fetch would resolve a broken action the moment "
        "@v0 moves; refusing to promote (#357)"
    )


def layout_ok(sha: str) -> str:
    return f"detect action layout present in the archive of {sha}"
