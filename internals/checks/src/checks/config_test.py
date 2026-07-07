"""Colocated unit test for the shared workflow paths (#321)."""
from checks.config import DOGFOOD_HELPERS_WORKFLOW, REUSABLE_WORKFLOW


def test_reusable_workflow_is_the_shipped_workflow_path():
    assert REUSABLE_WORKFLOW == ".github/workflows/testing-conventions.yml"


def test_dogfood_helpers_is_the_helpers_workflow_path():
    assert DOGFOOD_HELPERS_WORKFLOW == ".github/workflows/dogfood-github-helpers.yml"
