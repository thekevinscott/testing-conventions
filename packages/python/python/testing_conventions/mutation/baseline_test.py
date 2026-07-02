"""Unit tests for the mutation adapter's baseline check (#248)."""
from types import SimpleNamespace

import pytest

from testing_conventions.mutation.baseline import check_baseline


def _result(outcome, output=""):
    return SimpleNamespace(test_outcome=outcome, output=output)


def test_passes_when_the_unmutated_suite_survives(cosmic_ray):
    cosmic_ray.db.results = iter([("baseline", _result("survived"))])
    check_baseline({"module-path": ["."]})  # no raise


def test_raises_when_the_unmutated_suite_fails(cosmic_ray):
    cosmic_ray.db.results = iter([("baseline", _result("killed", output="E   assert 1 == 2"))])
    with pytest.raises(RuntimeError, match="did not pass unmutated"):
        check_baseline({"module-path": ["."]})
