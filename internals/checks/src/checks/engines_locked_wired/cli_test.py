"""Colocated unit tests for the engines-locked-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. The raise path is asserted against the propagated
exception's `.message` rather than importing `CheckFailed`.
"""
from checks.engines_locked_wired.cli import CI_ENGINE_LOCK, WORKFLOWS_DIR, cli

PINNED = "      - run: uv run --with-requirements .github/uv/engines.txt --no-project pytest\n"
FLOATING = "      - run: uv run --with pytest --no-project pytest\n"


def _dir_with(tmp_path, **workflows):
    d = tmp_path / "workflows"
    d.mkdir()
    for name, body in workflows.items():
        (d / f"{name}.yml").write_text(body)
    return d


def _lock(tmp_path):
    lock = tmp_path / "engines.txt"
    lock.write_text("pytest==9.1.1\n")
    return lock


def test_echoes_when_every_workflow_is_pinned(tmp_path, capsys):
    d = _dir_with(tmp_path, rust=PINNED, detect=PINNED)
    cli.callback(workflows_dir=str(d), lock=str(_lock(tmp_path)))
    assert "none floats" in capsys.readouterr().out


def test_raises_and_names_the_offending_file_and_engine(tmp_path):
    d = _dir_with(tmp_path, clean=PINNED, rust=FLOATING)
    try:
        cli.callback(workflows_dir=str(d), lock=str(_lock(tmp_path)))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "rust.yml" in error.message and "pytest" in error.message
        assert "clean.yml" not in error.message
    else:
        raise AssertionError("a workflow floating a `--with` engine must raise")


def test_raises_when_the_lock_is_missing(tmp_path):
    d = _dir_with(tmp_path, rust=PINNED)
    try:
        cli.callback(workflows_dir=str(d), lock=str(tmp_path / "absent.txt"))
    except Exception as error:  # noqa: BLE001
        assert "missing" in error.message
    else:
        raise AssertionError("a missing engine lock must raise")


def test_declares_the_arguments_defaulting_to_the_workflows_dir_and_engine_lock():
    argument, option = cli.params
    assert argument.name == "workflows_dir"
    assert argument.default == WORKFLOWS_DIR
    assert option.name == "lock"
    assert option.default == CI_ENGINE_LOCK
