"""Colocated unit tests for the engines-drift check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback`; the raise path is asserted against the
propagated exception's `.message`.
"""
from checks.engines_drift.cli import CI_ENGINE_LOCK, cli


def _write(tmp_path, name, body):
    path = tmp_path / name
    path.write_text(body)
    return str(path)


def test_echoes_when_the_lock_matches_latest(tmp_path, capsys):
    lock = _write(tmp_path, "lock.txt", "pytest==9.1.1\n")
    latest = _write(tmp_path, "latest.txt", "pytest==9.1.1\n")
    cli.callback(latest=latest, lock=lock)
    assert "no drift" in capsys.readouterr().out


def test_raises_and_names_the_drift(tmp_path):
    lock = _write(tmp_path, "lock.txt", "pytest==9.1.1\n")
    latest = _write(tmp_path, "latest.txt", "pytest==9.2.0\n")
    try:
        cli.callback(latest=latest, lock=lock)
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "pytest: 9.1.1 → 9.2.0" in error.message
    else:
        raise AssertionError("a drifted toolchain must raise")


def test_declares_a_required_latest_option_and_a_lock_defaulting_to_the_engine_lock():
    latest_opt = next(p for p in cli.params if p.name == "latest")
    lock_opt = next(p for p in cli.params if p.name == "lock")
    assert latest_opt.required is True
    assert latest_opt.type.exists is True  # click rejects a missing latest resolution up front
    assert lock_opt.default == CI_ENGINE_LOCK
