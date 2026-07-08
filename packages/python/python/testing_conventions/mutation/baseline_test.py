"""Unit tests for the mutation adapter's baseline check."""
from types import SimpleNamespace

import pytest

from testing_conventions.mutation.baseline import check_baseline


def _result(outcome, output=""):
    return SimpleNamespace(test_outcome=outcome, output=output)


def test_passes_when_the_unmutated_suite_survives(cosmic_ray):
    cosmic_ray.db.results = iter([("baseline", _result("survived"))])
    check_baseline({"module-path": ["."]})  # no raise


def test_passes_for_a_non_killed_outcome_sorting_below_killed(cosmic_ray):
    # "incompetent" < "killed" lexicographically but is not "killed"; only an exact
    # equality check leaves it un-raised (a ``<=`` comparison would wrongly raise).
    cosmic_ray.db.results = iter([("baseline", _result("incompetent"))])
    check_baseline({"module-path": ["."]})  # no raise


def test_raises_when_the_unmutated_suite_fails(cosmic_ray):
    # Build "killed" at runtime so it is a distinct object from the interned literal
    # the check compares against — an identity (``is``) check would miss it.
    killed = "".join(list("killed"))
    cosmic_ray.db.results = iter([("baseline", _result(killed, output="E   assert 1 == 2"))])
    with pytest.raises(RuntimeError, match="did not pass unmutated") as raised:
        check_baseline({"module-path": ["."]})
    # The cosmic-ray output must be carried into the message (``or ""``, not ``and ""``).
    assert "E   assert 1 == 2" in str(raised.value)
