import pytest

import hermetic


def test_hermetic_for_this_repos_own_caller_with_no_version():
    # Built at runtime, so it is equal to the constant without being the same interned object:
    # comparing the caller by identity would leave every real run on the published path.
    caller = "/".join(["thekevinscott", "testing-conventions"])
    assert hermetic.hermetic(caller, "") is True


@pytest.mark.parametrize("caller", ["someone/else", "zzz/after-this-repo"])
def test_not_hermetic_for_any_other_caller(caller):
    assert hermetic.hermetic(caller, "") is False


def test_an_explicit_version_wins_over_hermetic():
    assert hermetic.hermetic("thekevinscott/testing-conventions", "0.3.0") is False


def test_not_hermetic_when_the_caller_is_unknown():
    assert hermetic.hermetic("", "") is False
