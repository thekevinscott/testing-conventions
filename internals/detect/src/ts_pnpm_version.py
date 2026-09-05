from pathlib import Path

from read_package_json import read_package_json

PNPM_FLOOR = ">=11"


def _pnpm_version_pin(declared: str) -> str:
    """The `pnpm/action-setup` `version` input for a `packageManager` value: the consumer's own
    pin when it names pnpm — the only value the action accepts alongside such a pin — else
    [`PNPM_FLOOR`]. Never empty, so the workflow can read empty as a stale detect."""
    name, _, version = declared.partition("@")
    return version if name == "pnpm" else PNPM_FLOOR


def ts_pnpm_version(package_root: Path) -> str:
    """The pnpm version the reusable workflow should install for `package_root`."""
    return _pnpm_version_pin(read_package_json(package_root).get("packageManager", ""))
