"""Decide whether every red-path job carries both the `build-cli` edge and the
`download-hermetic-cli` step, bounded to its own YAML region so a neighbour's wiring satisfies
none of it."""
from __future__ import annotations

import re

from checks.utils.job_block import iter_job_blocks

# The direct-drive failure-path jobs: each runs the CLI by hand over a fixture and asserts its
# exit code, so each needs the HEAD binary the `uses:`-called jobs already download.
RED_PATH_JOBS = (
    "below-floor",
    "mutation-gate",
    "python-mutation-clean",
    "isolation-red",
    "packaging-red",
    "coverage-rust-red",
    "integration-lint-new-arms-trip",
    "packaging-package-root-red",
    "colocated-rust-red",
)

_NEEDS_BUILD_CLI = re.compile(r"^\s*needs: \[[^\]]*\bbuild-cli\b[^\]]*\]$", re.M)
_DOWNLOAD = "uses: ./.github/actions/download-hermetic-cli"


def unwired_jobs(text: str) -> list[str]:
    """The red-path jobs missing their hermetic wiring, in `RED_PATH_JOBS` order.

    A job is wired only when its own block carries both a `needs: [... build-cli ...]` edge and the
    `download-hermetic-cli` step; a job absent from the file is reported too.
    """
    blocks = dict(iter_job_blocks(text))
    missing = []
    for job in RED_PATH_JOBS:
        block = blocks.get(job)
        if block is None or not _NEEDS_BUILD_CLI.search(block) or _DOWNLOAD not in block:
            missing.append(job)
    return missing
