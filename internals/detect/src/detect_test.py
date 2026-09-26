import os
import runpy
import sys
from unittest.mock import patch

import pytest

import detect

DEFAULT_CONFIG = "testing-conventions.toml"
MISSING_ERROR = (
    "::error::detect.py takes an argument per input, in order: languages, scan_path, config, "
    "caller_repository, version. Missing: "
)


def command_line(languages="", scan_path=".", config=DEFAULT_CONFIG, caller_repository="", version=""):
    """A whole five-argument command line, one keyword per position."""
    return [languages, scan_path, config, caller_repository, version]


def run_main(argv, env=None):
    """Run `main` over `argv`, with `env` as the whole environment."""
    with patch.dict(os.environ, env or {}, clear=True):
        return detect.main(argv)


def test_main_scans_the_path_its_command_line_names(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.ts")
    write(tmp_path / "pkg" / "mod.py")
    assert run_main(command_line(scan_path="pkg")) == 0
    assert capsys.readouterr().out.startswith("languages='' under 'pkg' -> languages [\"python\"], ")


def test_main_restricts_the_scan_to_the_requested_languages(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main(command_line(languages='["typescript"]'))
    assert capsys.readouterr().out.startswith("languages='[\"typescript\"]' under '.' -> languages [], ")


def test_main_reads_the_requested_config(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "custom.toml", '[python]\nbuild_command = "make"\n')
    write(tmp_path / "mod.py")
    run_main(command_line(config="custom.toml"))
    assert "config custom.toml, build_command make," in capsys.readouterr().out


def test_main_builds_the_cli_from_head_for_this_repositorys_own_run(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main(command_line(caller_repository="thekevinscott/testing-conventions"))
    assert "cli_command ./hermetic-cli/testing-conventions," in capsys.readouterr().out


def test_main_takes_the_published_path_when_a_version_is_requested(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main(command_line(caller_repository="thekevinscott/testing-conventions", version="1.2.3"))
    assert "cli_command , " in capsys.readouterr().out


def test_main_takes_an_empty_argument_as_a_value_rather_than_an_absent_one(tmp_path, in_dir, write, capsys):
    # Empty is the action's own default for three of the five: it auto-detects every language,
    # takes the published CLI path, and pins no version.
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    assert run_main(command_line(languages="", caller_repository="", version="")) == 0
    out = capsys.readouterr().out
    assert out.startswith("languages='' under '.' -> languages [\"python\"], ")
    assert "cli_command , " in out


def test_main_takes_an_empty_scan_path_as_a_value_too(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    assert run_main(command_line(scan_path="")) == 0
    assert capsys.readouterr().out.startswith("languages='' under '' -> ")


def test_main_refuses_a_command_line_that_stops_short(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    assert run_main(["", "pkg"]) == 1


def test_main_names_every_argument_the_command_line_stops_short_of(tmp_path, in_dir, capsys):
    in_dir(tmp_path)
    run_main(["", "pkg"])
    assert capsys.readouterr().err == MISSING_ERROR + "config, caller_repository, version\n"


def test_main_refuses_an_empty_command_line_rather_than_scanning_the_working_directory(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    assert run_main([]) == 1
    captured = capsys.readouterr()
    assert captured.err == MISSING_ERROR + "languages, scan_path, config, caller_repository, version\n"
    assert captured.out == ""


def test_main_writes_no_outputs_when_an_argument_is_absent(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    receipt = write(tmp_path / "outputs.txt", "already=here\n")
    assert run_main(["", "."], {"GITHUB_OUTPUT": str(receipt)}) == 1
    assert receipt.read_text() == "already=here\n"


def test_main_ignores_an_argument_past_the_last_one_it_reads(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    assert run_main(command_line() + ["surplus"]) == 0
    assert capsys.readouterr().out.startswith("languages='' under '.' -> languages [\"python\"], ")


def test_main_appends_the_outputs_to_the_github_output_file(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    receipt = write(tmp_path / "outputs.txt", "already=here\n")
    run_main(command_line(), {"GITHUB_OUTPUT": str(receipt)})
    written = receipt.read_text()
    assert written.startswith("already=here\n")
    assert 'languages=["python"]\n' in written
    assert written.endswith("ts_mutation_adapter_args=\n")


def test_main_prints_the_outputs_with_no_github_output_file(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main(command_line())
    assert capsys.readouterr().out.endswith(", cli_command , ts_mutation_adapter_args \n")


def test_running_the_module_as_a_script_exits_with_mains_status(tmp_path, in_dir):
    in_dir(tmp_path)
    run_name = "".join(["__main", "__"])
    with patch.object(sys, "argv", [detect.__file__] + command_line()), patch.dict(os.environ, {}, clear=True):
        with pytest.raises(SystemExit) as exit_info:
            runpy.run_path(detect.__file__, run_name=run_name)
    assert exit_info.value.code == 0


def test_running_the_module_as_a_script_scans_the_path_its_command_line_names(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "pkg" / "mod.py")
    run_name = "".join(["__main", "__"])
    argv = [detect.__file__] + command_line(scan_path="pkg")
    with patch.object(sys, "argv", argv), patch.dict(os.environ, {}, clear=True):
        with pytest.raises(SystemExit):
            runpy.run_path(detect.__file__, run_name=run_name)
    assert capsys.readouterr().out.startswith("languages='' under 'pkg' -> languages [\"python\"], ")


def test_running_the_module_as_a_script_with_no_arguments_exits_nonzero(tmp_path, in_dir, capsys):
    in_dir(tmp_path)
    run_name = "".join(["__main", "__"])
    with patch.object(sys, "argv", [detect.__file__]), patch.dict(os.environ, {}, clear=True):
        with pytest.raises(SystemExit) as exit_info:
            runpy.run_path(detect.__file__, run_name=run_name)
    assert exit_info.value.code == 1
    assert capsys.readouterr().err.startswith(MISSING_ERROR)


@pytest.mark.parametrize("run_name", ["__init__", "detect"])
def test_running_the_module_under_any_other_name_leaves_main_uncalled(tmp_path, in_dir, run_name):
    in_dir(tmp_path)
    with patch.dict(os.environ, {}, clear=True):
        assert runpy.run_path(detect.__file__, run_name=run_name)["__name__"] == run_name
