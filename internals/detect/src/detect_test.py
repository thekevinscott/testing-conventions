import os
import runpy
from unittest.mock import patch

import pytest

import detect


def run_main(env):
    """Run `main` with `env` as the whole environment."""
    with patch.dict(os.environ, env, clear=True):
        return detect.main()


def test_main_scans_the_working_directory_by_default(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    assert run_main({}) == 0
    assert capsys.readouterr().out.startswith("languages='' under '.' -> languages [\"python\"], ")


def test_main_scans_the_requested_path(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.ts")
    write(tmp_path / "pkg" / "mod.py")
    run_main({"SCAN_PATH": "pkg"})
    assert capsys.readouterr().out.startswith("languages='' under 'pkg' -> languages [\"python\"], ")


def test_main_restricts_the_scan_to_the_requested_languages(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main({"LANGUAGES": '["typescript"]'})
    assert capsys.readouterr().out.startswith("languages='[\"typescript\"]' under '.' -> languages [], ")


def test_main_reads_the_requested_config(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "custom.toml", '[python]\nbuild_command = "make"\n')
    write(tmp_path / "mod.py")
    run_main({"CONFIG": "custom.toml"})
    assert "config custom.toml, build_command make," in capsys.readouterr().out


def test_main_builds_the_cli_from_head_for_this_repositorys_own_run(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main({"CALLER_REPOSITORY": "thekevinscott/testing-conventions"})
    assert "cli_command ./hermetic-cli/testing-conventions," in capsys.readouterr().out


def test_main_takes_the_published_path_when_a_version_is_requested(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    env = {"CALLER_REPOSITORY": "thekevinscott/testing-conventions", "VERSION": "1.2.3"}
    run_main(env)
    assert "cli_command , " in capsys.readouterr().out


def test_main_appends_the_outputs_to_the_github_output_file(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    receipt = write(tmp_path / "outputs.txt", "already=here\n")
    run_main({"GITHUB_OUTPUT": str(receipt)})
    written = receipt.read_text()
    assert written.startswith("already=here\n")
    assert 'languages=["python"]\n' in written
    assert written.endswith("ts_mutation_adapter_args=\n")


def test_main_prints_the_outputs_with_no_github_output_file(tmp_path, in_dir, write, capsys):
    in_dir(tmp_path)
    write(tmp_path / "mod.py")
    run_main({})
    assert capsys.readouterr().out.endswith(", cli_command , ts_mutation_adapter_args \n")


def test_running_the_module_as_a_script_exits_with_mains_status(tmp_path, in_dir):
    in_dir(tmp_path)
    run_name = "".join(["__main", "__"])
    with patch.dict(os.environ, {}, clear=True):
        with pytest.raises(SystemExit) as exit_info:
            runpy.run_path(detect.__file__, run_name=run_name)
    assert exit_info.value.code == 0


@pytest.mark.parametrize("run_name", ["__init__", "detect"])
def test_running_the_module_under_any_other_name_leaves_main_uncalled(tmp_path, in_dir, run_name):
    in_dir(tmp_path)
    with patch.dict(os.environ, {}, clear=True):
        assert runpy.run_path(detect.__file__, run_name=run_name)["__name__"] == run_name
