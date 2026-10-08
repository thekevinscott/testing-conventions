"""The output names an action manifest declares."""
from __future__ import annotations

import re


def declared_outputs(manifest: str) -> set[str]:
    """The names under an action manifest's `outputs:` mapping — empty when it declares none.

    Parsed with the stdlib against the manifest's fixed two-space shape, so the published
    manifest is read as the bytes GitHub resolves rather than through a YAML dependency.
    `[a-z0-9_]+`, never `[a-z_]+`: `e2e_attestation` and its two siblings carry a digit, and a
    pattern that drops them drops them from the declared side of a superset that must hold.
    """
    block = manifest.partition("\noutputs:\n")[2]
    next_key = re.search(r"^\S", block, re.M)
    bounded = block if next_key is None else block[: next_key.start()]
    return set(re.findall(r"^  ([a-z0-9_]+):", bounded, re.M))
