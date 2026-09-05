import build_command_language


def test_build_command_language_prefers_primary():
    derived = build_command_language.build_command_language("python", ["python", "typescript"])
    assert derived == "python"


def test_build_command_language_falls_back_to_the_sole_present_language():
    assert build_command_language.build_command_language("", ["python"]) == "python"


def test_build_command_language_empty_when_no_manifest_and_none_present():
    assert build_command_language.build_command_language("", []) == ""


def test_build_command_language_empty_when_no_manifest_and_ambiguous():
    assert build_command_language.build_command_language("", ["python", "typescript"]) == ""
