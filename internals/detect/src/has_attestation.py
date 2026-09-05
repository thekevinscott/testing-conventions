from pathlib import Path


def has_attestation(root: Path) -> bool:
    """True if committed e2e receipts sit at `root` — the package root, not the checkout root:
    a `.json` under `e2e-attestations/`, or the legacy single `e2e-attestation.json`."""
    if (root / "e2e-attestation.json").is_file():
        return True
    receipts = root / "e2e-attestations"
    return receipts.is_dir() and any(
        entry.suffix == ".json" and entry.is_file() for entry in receipts.iterdir()
    )
