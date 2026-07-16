"""Colocated unit tests for the engines-locked-wired decision (isolation — pure text in/out)."""
import pytest

from checks.engines_locked_wired.decide import CI_ENGINES, decide, floating_engines

PINNED = "      - run: uv run --with-requirements .github/uv/engines.txt --no-project pytest\n"


def test_a_pinned_with_requirements_workflow_passes():
    assert decide(PINNED)
    assert floating_engines(PINNED) == []


@pytest.mark.parametrize("engine", CI_ENGINES)
def test_a_bare_with_for_each_engine_fails(engine):
    text = f"      - run: uv run --with {engine} --no-project pytest\n"
    assert not decide(text)
    assert floating_engines(text) == [engine]


def test_with_requirements_is_not_read_as_a_floating_with():
    # The pinned form differs from the floating one only by the hyphen-vs-space after `--with`; a
    # regex that dropped the trailing space would match `--with-requirements` and wrongly fail.
    assert floating_engines("uv run --with-requirements .github/uv/engines.txt pytest") == []


def test_a_floating_engine_named_only_in_a_comment_is_ignored():
    # The prose that describes a run — e.g. rust.yml's "layers `--with coverage --with pytest`" —
    # is context, not an invocation; dropping comment lines keeps it from tripping the guard.
    comment = "      # below (`--with coverage --with pytest --with cosmic-ray`), and PYTHONPATH\n"
    assert decide(comment)
    assert floating_engines(comment) == []


def test_a_comment_does_not_mask_a_real_floating_run_on_another_line():
    text = (
        "      # historical note about --with pytest\n"
        "      - run: uv run --with cosmic-ray --no-project pytest\n"
    )
    assert not decide(text)
    assert floating_engines(text) == ["cosmic-ray"]


def test_every_floating_engine_on_one_line_is_listed():
    text = "      - run: uv run --with coverage --with pytest --with cosmic-ray --no-project cargo\n"
    assert floating_engines(text) == ["coverage", "pytest", "cosmic-ray"]


def test_decide_is_the_negation_of_floating_engines():
    floating = "uv run --with pytest x"
    assert decide(floating) is (floating_engines(floating) == [])
    assert decide(PINNED) is (floating_engines(PINNED) == [])
