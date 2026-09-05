"""Colocated unit tests for the CLI-invoking job finder (isolation — pure text in/out)."""
from checks.cli_node_engine_wired.cli_jobs import cli_jobs

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


def test_cli_jobs_names_only_the_jobs_that_invoke_the_cli():
    assert [name for name, _ in cli_jobs(WIRED)] == ["static"]


def test_cli_jobs_is_empty_when_no_job_invokes_the_cli():
    assert cli_jobs("jobs:\n  detect:\n    runs-on: ubuntu-latest\n") == []
