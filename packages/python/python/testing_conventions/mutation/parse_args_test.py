"""Unit tests for the mutation adapter's argument parsing (#248)."""
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
