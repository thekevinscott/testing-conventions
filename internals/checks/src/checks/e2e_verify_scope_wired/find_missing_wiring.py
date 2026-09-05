"""The e2e-verify scope/base/extra-roots wiring decision, block-scoped to the job's own YAML."""
from __future__ import annotations

import re
from typing import Optional

from checks.e2e_verify_scope_wired.extract_e2e_verify_block import extract_e2e_verify_block

_HAS_SCOPE_FLAG = re.compile(r"--scope")
_HAS_SCAN_PATH_FROM_INPUTS_SOURCE = re.compile(r"SCAN_PATH:\s*.*inputs\.source")
_HAS_BASE_FLAG = re.compile(r"--base")
_HAS_BASE_FROM_INPUTS_BASE = re.compile(r"BASE:\s*.*inputs\.base")
_HAS_PULL_REQUEST_GATE = re.compile(r"github\.event_name == 'pull_request'")
# The run step appends the detect-rendered `$EXTRA_SCOPE` / `$EXCLUDE` (repeated
# `--extra-scope`/`--exclude` arguments), and their env is wired from detect's outputs.
_HAS_EXTRA_SCOPE_ARG = re.compile(r"\$EXTRA_SCOPE")
_HAS_EXTRA_SCOPE_FROM_DETECT = re.compile(r"EXTRA_SCOPE:\s*.*e2e_extra_scope")
_HAS_EXCLUDE_ARG = re.compile(r"\$EXCLUDE")
_HAS_EXCLUDE_FROM_DETECT = re.compile(r"EXCLUDE:\s*.*e2e_exclude")

_SCOPE_ERROR = (
    "the e2e-verify job doesn't pass --scope naming inputs.source — the freshness walk "
    "is scoped to the derived package_root instead, which can be broader than what the "
    "caller's own source input names (#294)"
)
_BASE_ERROR = (
    "the e2e-verify job doesn't pass --base naming inputs.base — the freshness walk "
    "is history-absolute instead of scoped to this branch's `<base>..HEAD` diff, which "
    "reds unrelated PRs on a squash-merging repo (#319)"
)
_GATE_ERROR = (
    "the e2e-verify job isn't gated to `github.event_name == 'pull_request'` — its "
    "--base diff needs a base ref, so it must ride the same pull-request gate as the "
    "other diff-scoped jobs (#319)"
)
_EXTRA_SCOPE_ERROR = (
    "the e2e-verify job doesn't append $EXTRA_SCOPE from detect's e2e_extra_scope output — a "
    "shared source tree beside the package (a native core bound into several bindings) can't "
    "join the freshness walk, so a core-only PR leaves the binding attestation falsely fresh "
    "(#333)"
)
_EXCLUDE_ERROR = (
    "the e2e-verify job doesn't append $EXCLUDE from detect's e2e_exclude output — a "
    "feature-gated subtree of an extra root (a core cli/ compiled out of the bindings) can't "
    "be carved back out, so a change only under it would falsely stale the attestation (#333)"
)


def find_missing_wiring(workflow_text: str) -> Optional[str]:
    """None if the e2e-verify job passes --scope naming inputs.source and --base naming
    inputs.base under a pull-request gate, and appends the detect-rendered $EXTRA_SCOPE /
    $EXCLUDE arguments from detect's outputs; else the first error message."""
    block = extract_e2e_verify_block(workflow_text)
    if not _HAS_SCOPE_FLAG.search(block) or not _HAS_SCAN_PATH_FROM_INPUTS_SOURCE.search(block):
        return _SCOPE_ERROR
    if not _HAS_BASE_FLAG.search(block) or not _HAS_BASE_FROM_INPUTS_BASE.search(block):
        return _BASE_ERROR
    if not _HAS_PULL_REQUEST_GATE.search(block):
        return _GATE_ERROR
    if not _HAS_EXTRA_SCOPE_ARG.search(block) or not _HAS_EXTRA_SCOPE_FROM_DETECT.search(block):
        return _EXTRA_SCOPE_ERROR
    if not _HAS_EXCLUDE_ARG.search(block) or not _HAS_EXCLUDE_FROM_DETECT.search(block):
        return _EXCLUDE_ERROR
    return None
