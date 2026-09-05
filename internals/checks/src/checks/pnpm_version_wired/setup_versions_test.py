"""Colocated unit tests for the pnpm version read (isolation — pure text in/out)."""
from checks.pnpm_version_wired.setup_versions import setup_versions

GUARDED = "${{ needs.detect.outputs.ts_pnpm_version || '>=11' }}"


def step(version: str) -> str:
    return f"      - uses: pnpm/action-setup@v5\n        with:\n          version: {version}\n"


def test_setup_versions_reads_each_steps_version_in_order():
    assert setup_versions(step('">=11"') + step(GUARDED)) == ['">=11"', GUARDED]


def test_setup_versions_ignores_a_version_belonging_to_a_later_step():
    # A pnpm step that sets no version must contribute nothing rather than borrow the next
    # step's — otherwise an unpinned step reads as wired.
    text = "      - uses: pnpm/action-setup@v5\n      - uses: actions/setup-node@v6\n        with:\n          version: 24\n"
    assert setup_versions(text) == []


def test_setup_versions_finds_nothing_without_a_pnpm_step():
    assert setup_versions("      - uses: actions/checkout@v6\n") == []


def test_setup_versions_reads_a_step_whose_uses_follows_its_if_line():
    # The real steps open with `- if:` and carry `uses:` a line later, so the step chunk — not
    # the `uses:` line — is what has to be searched.
    text = (
        "      - if: matrix.language == 'typescript'\n"
        "        uses: pnpm/action-setup@v5\n"
        "        with:\n"
        f"          version: {GUARDED}\n"
    )
    assert setup_versions(text) == [GUARDED]


def test_setup_versions_ignores_a_version_declared_before_any_step():
    # The workflow declares its own `version:` input far above the jobs. Lines preceding the
    # first step belong to no step, so they must not be read as one step's pin.
    text = "on:\n  workflow_call:\n    inputs:\n      version:\n        type: string\n"
    assert setup_versions(text) == []
