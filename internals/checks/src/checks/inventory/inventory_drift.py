"""Whether the authored check inventory still agrees with the shipped workflow and CLI — pure."""

from __future__ import annotations

from collections.abc import Iterable, Mapping


def inventory_drift(
    entries: Iterable[Mapping[str, object]],
    job_names: Iterable[str],
    cli_commands: Iterable[str],
    gate_names: Iterable[str],
) -> list[str]:
    """Every way `entries` disagrees with the three surfaces that are facts rather than opinions."""
    raise NotImplementedError
