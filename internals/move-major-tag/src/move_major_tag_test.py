"""Unit tests for the entry point: the environment it reads and the report it prints.

`advance` is the one collaborator, patched on this module in a fixture; the environment is set
with `patch.dict`, so each test reads exactly the variables the workflow sets.
"""
import os
import runpy
from unittest.mock import patch

import pytest

import move_major_tag as m


@pytest.fixture
def advance():
    """Patch the orchestration and yield the mock, answering "advance" unless a test says otherwise."""
    with patch.object(m, "advance") as patched:
        patched.return_value = "advance"
        yield patched


def test_main_reports_a_missing_sha_as_an_error(capsys):
    with patch.dict(os.environ, {}, clear=True):
        assert m.main() == 1
    assert capsys.readouterr().out == (
        "::error::SHA is required (the released commit to advance the tag to)\n"
    )


def test_main_treats_a_whitespace_only_sha_as_missing():
    with patch.dict(os.environ, {"SHA": "   "}, clear=True):
        assert m.main() == 1


def test_main_advances_the_default_tag_when_none_is_named(advance):
    with patch.dict(os.environ, {"SHA": "newsha"}, clear=True):
        assert m.main() == 0
    advance.assert_called_once_with("v0", "newsha")


def test_main_advances_the_named_tag(advance):
    with patch.dict(os.environ, {"SHA": "newsha", "TAG": "v1"}, clear=True):
        m.main()
    advance.assert_called_once_with("v1", "newsha")


def test_main_falls_back_to_the_default_tag_when_tag_is_whitespace(advance):
    with patch.dict(os.environ, {"SHA": "newsha", "TAG": "  "}, clear=True):
        m.main()
    advance.assert_called_once_with("v0", "newsha")


def test_main_strips_surrounding_whitespace_from_the_sha(advance):
    with patch.dict(os.environ, {"SHA": "  newsha  "}, clear=True):
        m.main()
    advance.assert_called_once_with("v0", "newsha")


def test_main_strips_surrounding_whitespace_from_the_tag(advance):
    with patch.dict(os.environ, {"SHA": "newsha", "TAG": "  v1  "}, clear=True):
        m.main()
    advance.assert_called_once_with("v1", "newsha")


def test_main_reports_an_advance(advance, capsys):
    with patch.dict(os.environ, {"SHA": "newsha"}, clear=True):
        assert m.main() == 0
    assert capsys.readouterr().out == "advanced v0 -> newsha\n"


def test_main_reports_a_bootstrap(advance, capsys):
    advance.return_value = "bootstrap"
    with patch.dict(os.environ, {"SHA": "deadbeef"}, clear=True):
        assert m.main() == 0
    assert capsys.readouterr().out == "v0 did not exist yet; bootstrapped it at deadbeef\n"


def test_main_reports_a_noop(advance, capsys):
    advance.return_value = "noop"
    with patch.dict(os.environ, {"SHA": "oldsha"}, clear=True):
        assert m.main() == 0
    assert capsys.readouterr().out == "v0 is already at or ahead of oldsha; nothing to do\n"


def test_running_the_module_as_a_script_exits_with_mains_status():
    # Built at runtime, so it is equal to "__main__" without being the same interned object:
    # comparing run names by identity would leave the guard dead for a caller that does the same.
    run_name = "".join(["__main", "__"])
    with patch.dict(os.environ, {}, clear=True):
        with pytest.raises(SystemExit) as exit_info:
            runpy.run_path(m.__file__, run_name=run_name)
    assert exit_info.value.code == 1


@pytest.mark.parametrize("run_name", ["__init__", "move_major_tag"])
def test_running_the_module_under_any_other_name_leaves_main_uncalled(run_name):
    with patch.dict(os.environ, {}, clear=True):
        assert runpy.run_path(m.__file__, run_name=run_name)["__name__"] == run_name
