"""Colocated unit tests for detect.

Unit-level: the pure helpers exercised in isolation (no filesystem, no mocks). The orchestration
(`compute_outputs`) is covered by the integration suite with the filesystem boundary mocked, and
the whole script end-to-end by the e2e suite against a real tree — both under `tests/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import detect  # noqa: E402


def test_eligible_empty_restrictor_allows_any_language():
    assert detect.eligible("", "python") is True


def test_eligible_empty_array_allows_any_language():
    assert detect.eligible("[]", "rust") is True


def test_eligible_named_language_is_in_scope():
    assert detect.eligible('["python"]', "python") is True


def test_eligible_unnamed_language_is_excluded():
    assert detect.eligible('["python"]', "rust") is False


def test_to_json_is_compact():
    assert detect._to_json(["python", "rust"]) == '["python","rust"]'


def test_to_json_empty_is_brackets():
    assert detect._to_json([]) == "[]"
