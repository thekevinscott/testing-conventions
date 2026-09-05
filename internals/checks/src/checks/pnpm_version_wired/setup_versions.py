"""The `version:` values of a workflow's `pnpm/action-setup` steps."""
from __future__ import annotations

from checks.pnpm_version_wired.pnpm_steps import pnpm_steps


def setup_versions(text: str) -> list[str]:
    """The `version:` value of every `pnpm/action-setup` step in `text`, in file order."""
    return [
        stripped.removeprefix("version:").strip()
        for chunk in pnpm_steps(text)
        for stripped in (line.strip() for line in chunk)
        if stripped.startswith("version:")
    ]
