"""The packaging gate/provision/build/crate-scan wiring decision, block-scoped to the job."""
from __future__ import annotations

import re
from typing import Optional

from checks.packaging_build_wired.extract_packaging_block import extract_packaging_block

# The `if:` gate must run the job when detect derived a build, not only for a committed
# dist / uploaded artifact — matched by the job-level alternation that ORs `packaging_build`
# with `packaging_dist`, so a build-step's own `if: ... packaging_build != ''` doesn't count.
_HAS_BUILD_GATE = re.compile(
    r"packaging_build != '' \|\| needs\.detect\.outputs\.packaging_dist == 'true'"
)
_HAS_LANGUAGE_PROVISION = re.compile(r"needs\.detect\.outputs\.packaging_language")
_HAS_BUILD_EVAL = re.compile(r'eval "\$PACKAGING_BUILD"')
_HAS_BUILD_FROM_DETECT = re.compile(r"PACKAGING_BUILD:\s*.*needs\.detect\.outputs\.packaging_build")
# The scan must reach `target/package`, where `cargo package` writes the crate.
_HAS_CRATE_SCAN = re.compile(r"target/package")

_GATE_ERROR = (
    "the packaging job's `if:` doesn't run on detect's packaging_build output — a native "
    "package whose manifest states a build (uv build / <pm> pack / cargo package) but ships "
    "no committed dist/ and no uploaded artifact is skipped, so the gate never runs (#335)"
)
_PROVISION_ERROR = (
    "the packaging job doesn't provision the toolchain for detect's packaging_language — the "
    "derived build has no cargo/uv/node to run under, so it fails before producing a "
    "distribution to scan (#335)"
)
_BUILD_ERROR = (
    "the packaging job doesn't run the derived packaging_build (`eval \"$PACKAGING_BUILD\"` "
    "wired from detect's packaging_build output) before scanning — no distribution is built, "
    "so a package with no committed dist/ has nothing to scan (#335)"
)
_CRATE_SCAN_ERROR = (
    "the packaging job doesn't scan target/package — a `cargo package` crate is written there, "
    "not under dist/, so a built crate is never inspected for shipped test files (#335)"
)


def find_missing_wiring(workflow_text: str) -> Optional[str]:
    """None if the packaging job gates on packaging_build, provisions packaging_language, runs
    the derived packaging_build, and scans target/package; else the first error message."""
    block = extract_packaging_block(workflow_text)
    if not _HAS_BUILD_GATE.search(block):
        return _GATE_ERROR
    if not _HAS_LANGUAGE_PROVISION.search(block):
        return _PROVISION_ERROR
    if not _HAS_BUILD_EVAL.search(block) or not _HAS_BUILD_FROM_DETECT.search(block):
        return _BUILD_ERROR
    if not _HAS_CRATE_SCAN.search(block):
        return _CRATE_SCAN_ERROR
    return None
