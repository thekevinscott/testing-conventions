"""Unit tests for the mutation adapter's argument parsing."""
import pytest

from testing_conventions.mutation.parse_args import parse_args


def test_parses_out_and_repeated_module():
    args = parse_args(["--out", "/tmp/r.json", "--module", "a.py", "--module", "b.py"])
    assert args.out == "/tmp/r.json"
    assert args.modules == ["a.py", "b.py"]


def test_modules_default_to_empty():
    args = parse_args(["--out", "/tmp/r.json"])
    assert args.out == "/tmp/r.json"
    assert args.modules == []


def test_out_is_required():
    with pytest.raises(SystemExit):
        parse_args(["--module", "a.py"])


def test_parses_repeated_lines():
    args = parse_args(["--out", "r.json", "--lines", "a.py:1-2", "--lines", "b.py:4-4"])
    assert args.lines == ["a.py:1-2", "b.py:4-4"]


def test_lines_default_to_empty():
    assert parse_args(["--out", "r.json"]).lines == []
