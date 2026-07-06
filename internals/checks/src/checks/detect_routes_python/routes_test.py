"""Colocated unit tests for the routes-python decision (isolation — pure, no I/O)."""
from checks.detect_routes_python.routes import routes_python


def test_true_when_python_is_the_only_language():
    assert routes_python('["python"]') is True


def test_true_when_python_is_alongside_rust():
    assert routes_python('["python","rust"]') is True


def test_false_when_python_is_absent():
    assert routes_python('["rust"]') is False


def test_false_on_an_empty_array():
    assert routes_python("[]") is False


def test_false_on_malformed_json():
    assert routes_python("not json") is False


def test_false_on_a_non_list_json_value():
    assert routes_python('{"python": true}') is False


def test_false_on_empty_string():
    assert routes_python("") is False
