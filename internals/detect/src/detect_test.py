"""Colocated unit tests for detect.

Unit-level: the pure helpers exercised in isolation (no filesystem, no mocks). The orchestration
(`compute_outputs`) is covered by the integration suite with the filesystem boundary mocked, and
the whole script end-to-end by the e2e suite against a real tree — both under `tests/`.
"""
import detect


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


def test_package_manager_from_field_extracts_the_name():
    assert detect._package_manager_from_field("pnpm@8.6.0") == "pnpm"


def test_package_manager_from_field_empty_is_none():
    assert detect._package_manager_from_field("") is None


def test_build_command_language_prefers_primary():
    assert detect.build_command_language("python", ["python", "typescript"]) == "python"


def test_build_command_language_falls_back_to_the_sole_present_language():
    assert detect.build_command_language("", ["python"]) == "python"


def test_build_command_language_empty_when_no_manifest_and_none_present():
    assert detect.build_command_language("", []) == ""


def test_build_command_language_empty_when_no_manifest_and_ambiguous():
    assert detect.build_command_language("", ["python", "typescript"]) == ""


def test_hermetic_for_this_repos_own_caller_with_no_version():
    assert detect.hermetic("thekevinscott/testing-conventions", "") is True


def test_not_hermetic_for_any_other_caller():
    assert detect.hermetic("someone/else", "") is False


def test_an_explicit_version_wins_over_hermetic():
    assert detect.hermetic("thekevinscott/testing-conventions", "0.3.0") is False


def test_not_hermetic_when_the_caller_is_unknown():
    assert detect.hermetic("", "") is False
