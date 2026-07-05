"""Colocated unit tests for check_detect_routes_python.

Unit-level: the pure `routes_python` decision over crafted `isolation_languages` JSON strings,
in isolation (no file reads, no subprocess, no network). `main` and the `__main__` guard are
covered by the e2e suite.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_detect_routes_python as m  # noqa: E402


def test_python_only_routes_in():
    assert m.routes_python('["python"]') is None


def test_python_with_rust_routes_in():
    assert m.routes_python('["python","rust"]') is None


def test_rust_only_does_not_route_python_in():
    assert m.routes_python('["rust"]') == (
        'the detect action did not route Python into isolation_languages (got: ["rust"])'
    )


def test_empty_array_does_not_route_python_in():
    assert m.routes_python("[]") == (
        "the detect action did not route Python into isolation_languages (got: [])"
    )


def test_malformed_json_does_not_route_python_in():
    assert m.routes_python("not json") == (
        "the detect action did not route Python into isolation_languages (got: not json)"
    )


def test_non_list_json_does_not_route_python_in():
    # A JSON object, not an array — routing requires membership in a list.
    assert m.routes_python('{"python": true}') is not None


def test_empty_string_does_not_route_python_in():
    assert m.routes_python("") is not None
