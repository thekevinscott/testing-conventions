"""The red-path-hermetic-wired decision — repo-only (#379, #356).

The self-test's direct-drive failure-path jobs assert the CLI's exit code over a fixture by hand
(a `uses:` call can't assert a non-zero exit). #356 made every `uses:`-called job hermetic — it
runs the CLI built from HEAD — but these direct-drive jobs sat outside that mechanism and drove
the published `npx testing-conventions`, so the merge gate still depended on npm-latest for their
assertions (#379). Each must instead download the `hermetic-cli` artifact the caller's `build-cli`
job stages and run the HEAD binary, exactly as the `uses:`-called jobs do.

The decision confirms every red-path job carries both halves of that wiring: a
`needs: [... build-cli ...]` edge (so the artifact exists before the job runs) and the
`./.github/actions/download-hermetic-cli` step (so the binary is fetched and made executable).
`iter_job_blocks` bounds each job to its own region so a neighbour's edge or download can't satisfy
a job that's missing its own.
"""
from __future__ import annotations

import re

from checks.utils.job_block import iter_job_blocks

# The direct-drive failure-path jobs (#379): each runs the CLI by hand over a fixture and asserts
# its exit code, so each needs the HEAD binary the `uses:`-called jobs already download.
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
