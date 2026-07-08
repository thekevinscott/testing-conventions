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


def _parse_github_output(text):
    """Parse a GITHUB_OUTPUT file the way the Actions runner does: `name=value` lines
    plus the heredoc `name<<DELIM` / body / `DELIM` form for multi-line values."""
    result = {}
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if "<<" in line and "=" not in line.split("<<", 1)[0]:
            name, delim = line.split("<<", 1)
            i += 1
            body = []
            while i < len(lines) and lines[i] != delim:
                body.append(lines[i])
                i += 1
            result[name] = "\n".join(body)
            i += 1  # skip the closing delimiter
        elif "=" in line:
            name, value = line.split("=", 1)
            result[name] = value
            i += 1
        else:
            i += 1  # blank/trailing line
    return result


def test_render_github_output_writes_single_line_values_as_name_equals_value():
    rendered = detect.render_github_output({"languages": '["python"]', "package_root": "."})
    assert _parse_github_output(rendered) == {"languages": '["python"]', "package_root": "."}


def test_render_github_output_round_trips_a_multiline_value():
    # A multi-line value (a legal TOML `"""…"""` build_command) must round-trip through
    # GITHUB_OUTPUT via the heredoc `name<<EOF` form; a raw `name=value` line would let
    # the embedded newline split it into a bogus second output (#396).
    outputs = {
        "build_command": "cp a.tmpl a.py\ncp b.tmpl b.py",
        "languages": '["python"]',
    }
    rendered = detect.render_github_output(outputs)
    assert _parse_github_output(rendered) == outputs


def test_an_explicit_version_wins_over_hermetic():
    assert detect.hermetic("thekevinscott/testing-conventions", "0.3.0") is False


def test_not_hermetic_when_the_caller_is_unknown():
    assert detect.hermetic("", "") is False
