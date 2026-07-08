"""Unit tests for the mutation adapter's baseline check."""
from types import SimpleNamespace

import pytest

from testing_conventions.mutation.baseline import check_baseline


def _result(outcome, output=""):
    return SimpleNamespace(test_outcome=outcome, output=output)


def test_returns_the_observed_runtime_when_the_suite_passes(cosmic_ray):
    # A passing baseline reports ``survived``; the check returns the clean run's wall-clock
    # seconds (later tick minus earlier tick), which scopes every mutant's timeout. The injected
    # clock pins the elapsed to a span where subtraction and modulo diverge (25.0 - 10.0 = 15.0,
    # not 25.0 % 10.0 = 5.0), so the ``-`` cannot be flipped to ``%``. ``survived`` is built at
    # runtime so it is a distinct object from the interned literal the check compares against: the
    # comparison is equality (``!=``), not identity (``is not``), and this passing outcome returns
    # rather than raising.
    survived = "".join(list("survived"))
    cosmic_ray.db.results = iter([("baseline", _result(survived))])
    ticks = iter([10.0, 25.0])
    observed = check_baseline({"module-path": ["."]}, clock=lambda: next(ticks))
    assert observed == 15.0


def test_reads_an_enum_like_outcomes_value(cosmic_ray):
    # cosmic-ray's ``TestOutcome`` is enum-like; the check reads ``.value`` before comparing,
    # so an enum-shaped ``survived`` passes exactly as the bare string does.
    cosmic_ray.db.results = iter([("baseline", _result(SimpleNamespace(value="survived")))])
    assert isinstance(check_baseline({"module-path": ["."]}), float)


def test_raises_when_the_unmutated_suite_fails(cosmic_ray):
    # A failing suite reports ``killed`` (the no-op baseline "died" on the already-red suite).
    # Build "killed" at runtime so it is a distinct object from the interned literal the check
    # compares against — an identity (``is``) check would miss it.
    killed = "".join(list("killed"))
    cosmic_ray.db.results = iter([("baseline", _result(killed, output="E   assert 1 == 2"))])
    with pytest.raises(RuntimeError, match="did not pass unmutated") as raised:
        check_baseline({"module-path": ["."]})
    # The failing outcome and the cosmic-ray output are both carried into the message.
    assert "killed" in str(raised.value)
    assert "E   assert 1 == 2" in str(raised.value)


def test_raises_when_the_baseline_times_out(cosmic_ray):
    # A suite too slow for its budget times out — cosmic-ray records no usable outcome
    # (``test_outcome`` is ``None``). The guard must fail loudly, not pass: otherwise every
    # mutant times out, ``normalize`` drops them all, and an empty survivor set false-greens.
    cosmic_ray.db.results = iter([("baseline", _result(None, output="timed out"))])
    with pytest.raises(RuntimeError, match="did not pass unmutated") as raised:
        check_baseline({"module-path": ["."]})
    assert "None" in str(raised.value)
    assert "timed out" in str(raised.value)


def test_raises_when_the_baseline_is_incompetent(cosmic_ray):
    # Only ``survived`` is a pass. An ``incompetent`` baseline — abnormal for unmutated code the
    # interpreter should accept — is untrustworthy, so it raises rather than slipping through
    # (the old guard raised only on ``killed``).
    cosmic_ray.db.results = iter([("baseline", _result("incompetent"))])
    with pytest.raises(RuntimeError, match="did not pass unmutated"):
        check_baseline({"module-path": ["."]})
