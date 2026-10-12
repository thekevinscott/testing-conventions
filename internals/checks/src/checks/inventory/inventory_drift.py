"""Whether the authored check inventory still agrees with the shipped workflow and CLI — pure."""

from __future__ import annotations

from collections.abc import Iterable, Mapping

from checks.inventory.claimed_names import claimed_names
from checks.inventory.surface_drift import surface_drift


def inventory_drift(
    entries: Iterable[Mapping[str, object]],
    job_names: Iterable[str],
    cli_commands: Iterable[str],
    gate_names: Iterable[str],
) -> list[str]:
    """Every way `entries` disagrees with the three surfaces that are facts rather than opinions.

    The three arrive as arguments because the inventory is held to the *shipped* workflow and
    binary: reading them here would make this a discovery problem, and a stale local build would
    then report drift that does not exist. The gates input comes first because it names the checks,
    so a disagreement there explains the other two.
    """
    listed = list(entries)
    return [
        *surface_drift("the gates input", claimed_names(listed, "gate"), gate_names),
        *surface_drift("the workflow", claimed_names(listed, "jobs"), job_names),
        *surface_drift("the CLI", claimed_names(listed, "command"), cli_commands),
    ]
