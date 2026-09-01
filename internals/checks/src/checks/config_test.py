"""Colocated unit tests for the shared constants.

Each literal is asserted where it is defined, so an edit that changes one without changing its
consumers co-changes this file — and a string mutant on it dies here.
"""
from checks.config import (
    DOGFOOD_WORKFLOW,
    HERMETIC_CLI,
    PY_FIXTURE_CONFIG,
    PY_FIXTURE_PACKAGE_ROOT,
    PY_FIXTURE_PYTHON_ENV,
    REUSABLE_WORKFLOW,
    SELFTEST_WORKFLOW,
    TS_FIXTURE_CONFIG,
    TS_FIXTURE_PACKAGE_MANAGER,
    TS_FIXTURE_PACKAGE_ROOT,
    TS_FIXTURE_PROVISION_RUST,
)


def test_workflow_paths_are_the_shipped_and_repo_only_callers():
    assert REUSABLE_WORKFLOW == ".github/workflows/testing-conventions.yml"
    assert SELFTEST_WORKFLOW == ".github/workflows/testing-conventions-selftest.yml"
    assert DOGFOOD_WORKFLOW == ".github/workflows/dogfood.yml"


def test_ts_fixture_expectations_are_the_monorepo_ts_package():
    assert TS_FIXTURE_PACKAGE_ROOT == ".github/selftest/monorepo/packages/ts"
    assert TS_FIXTURE_PACKAGE_MANAGER == "npm"
    assert TS_FIXTURE_PROVISION_RUST == "false"
    assert TS_FIXTURE_CONFIG == ".github/selftest/monorepo/packages/ts/testing-conventions.toml"


def test_py_fixture_expectations_are_the_monorepo_py_package():
    assert PY_FIXTURE_PACKAGE_ROOT == ".github/selftest/monorepo/packages/py"
    assert PY_FIXTURE_PYTHON_ENV == "uv"
    assert PY_FIXTURE_CONFIG == ".github/selftest/monorepo/packages/py/testing-conventions.toml"


def test_hermetic_cli_is_the_downloaded_binary_spliced_ahead_of_a_subcommand():
    assert HERMETIC_CLI == ["./hermetic-cli/testing-conventions"]
