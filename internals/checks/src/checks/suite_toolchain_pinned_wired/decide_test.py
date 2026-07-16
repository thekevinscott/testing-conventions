"""Colocated unit tests for the suite-toolchain-pinned-wired decision (isolation — pure text)."""
import pytest

from checks.suite_toolchain_pinned_wired.decide import SUITE_ENGINES, decide, unpinned_engines

PINNED = '          uv pip install "coverage==7.15.2" "pytest==9.1.1" testing-conventions\n'
FLOATING = "          uv pip install coverage pytest testing-conventions\n"


def test_a_pinned_install_line_passes():
    assert decide(PINNED)
    assert unpinned_engines(PINNED) == []


def test_a_fully_floating_install_line_fails():
    assert not decide(FLOATING)
    assert unpinned_engines(FLOATING) == ["coverage", "pytest"]


@pytest.mark.parametrize("engine", SUITE_ENGINES)
def test_one_floating_engine_beside_a_pinned_one_fails(engine):
    other = next(e for e in SUITE_ENGINES if e != engine)
    line = f'          uv pip install "{other}==1.0" {engine} testing-conventions\n'
    assert not decide(line)
    assert unpinned_engines(line) == [engine]


def test_a_floating_range_is_not_an_exact_pin():
    # `>=` still resolves to whatever the index serves above the floor — not reproducible.
    line = '          uv pip install "coverage>=7" "pytest==9.1.1" testing-conventions\n'
    assert not decide(line)
    assert unpinned_engines(line) == ["coverage"]


def test_testing_conventions_itself_need_not_be_pinned():
    # It is the package under test (published wheel / HEAD build), not a third-party engine.
    assert decide(PINNED)  # PINNED leaves `testing-conventions` bare, and that is fine


def test_a_workflow_that_never_installs_the_engines_fails_non_vacuously():
    # No coverage/pytest anywhere → the positive half (both present, pinned) is unmet.
    assert not decide("          uv pip install testing-conventions\n")


@pytest.mark.parametrize("present", SUITE_ENGINES)
def test_one_engine_pinned_with_the_other_absent_fails(present):
    # One engine pinned, the other named nowhere (not even floating), so `unpinned_engines` is
    # empty and the positive half alone decides: *every* engine must be pinned, not just one.
    line = f'          uv pip install "{present}==1.0" testing-conventions\n'
    assert unpinned_engines(line) == []
    assert not decide(line)


def test_an_install_mentioned_only_in_a_comment_is_ignored():
    comment = "          # historically this ran `uv pip install coverage pytest`\n"
    assert unpinned_engines(comment) == []
    assert not decide(comment)  # still fails: nothing is actually pinned


def test_every_install_line_must_pin_not_just_one():
    text = PINNED + FLOATING
    assert not decide(text)
    assert unpinned_engines(text) == ["coverage", "pytest"]
