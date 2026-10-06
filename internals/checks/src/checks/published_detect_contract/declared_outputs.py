"""The output names an action manifest declares."""
from __future__ import annotations


def declared_outputs(manifest: str) -> set[str]:
    """The names under an action manifest's `outputs:` mapping — empty when it declares none.

    Parsed with the stdlib against the manifest's fixed two-space shape, so the published
    manifest is read as the bytes GitHub resolves rather than through a YAML dependency.
    """
    raise NotImplementedError
