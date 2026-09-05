"""Colocated unit tests for the node-floor violations (isolation — pure text in/out)."""
from checks.cli_node_engine_wired.violations import violations

WIRED = """\
jobs:
  detect:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
  static:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/setup-node@v6
        with:
          node-version: 24
      - name: Check colocated test
        run: ${CLI_COMMAND:-npx -y "testing-conventions${VERSION:+@$VERSION}"} unit colocated-test
"""

GATED = WIRED.replace(
    "      - uses: actions/setup-node@v6",
    "      - if: matrix.language == 'typescript'\n        uses: actions/setup-node@v6",
)

TWO_NODES = WIRED.replace(
    "      - name: Check colocated test",
    "      - uses: actions/setup-node@v6\n        with:\n          node-version: 20\n"
    "      - name: Check colocated test",
)


def test_a_job_pinning_exactly_the_floor_is_no_violation():
    assert violations(WIRED, 24) == []


def test_a_job_with_no_setup_node_is_a_violation():
    text = WIRED.replace("      - uses: actions/setup-node@v6\n        with:\n          node-version: 24\n", "")
    assert violations(text, 24) == ["`static` invokes the CLI with no unconditional `setup-node` step"]


def test_a_setup_node_gated_by_an_if_does_not_count():
    assert violations(GATED, 24) == ["`static` invokes the CLI with no unconditional `setup-node` step"]


def test_a_job_pinning_above_the_floor_is_no_violation():
    assert violations(WIRED.replace("node-version: 24", "node-version: 26"), 24) == []


def test_a_job_pinning_below_the_floor_is_a_violation():
    text = WIRED.replace("node-version: 24", "node-version: 22")
    assert violations(text, 24) == ["`static` pins node 22, below the floor of 24"]


def test_the_highest_pinned_node_in_a_job_decides():
    assert violations(TWO_NODES, 24) == []
    assert violations(TWO_NODES, 26) == ["`static` pins node 24, below the floor of 26"]


def test_a_job_that_invokes_no_cli_needs_no_setup_node():
    assert violations("jobs:\n  detect:\n    runs-on: ubuntu-latest\n", 24) == []
