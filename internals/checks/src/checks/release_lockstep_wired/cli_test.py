"""Colocated unit tests for the release-lockstep-wired check."""
from checks.release_lockstep_wired.cli import RELEASE_CONFIG, cli

LOCKSTEP = """
[[package]]
name = "tc-rust"
kind = "crates"
path = "packages/rust"
globs = ["packages/rust/**"]

[[package]]
name = "tc-npm"
kind = "npm"
depends_on = ["tc-rust"]
globs = ["packages/node/**", "packages/rust/**"]

[[package]]
name = "tc-py"
kind = "pypi"
depends_on = ["tc-rust"]
globs = ["packages/python/**", "packages/rust/**"]
"""


def test_echoes_on_a_lockstep_config(tmp_path, capsys):
    config = tmp_path / "putitoutthere.toml"
    config.write_text(LOCKSTEP)
    cli.callback(config=str(config))
    assert "advance together" in capsys.readouterr().out


def test_raises_when_a_half_drops_the_crate_glob(tmp_path):
    config = tmp_path / "putitoutthere.toml"
    config.write_text(LOCKSTEP.replace(', "packages/rust/**"]\n', "]\n", 1))
    try:
        cli.callback(config=str(config))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "drift apart" in error.message
        assert "tc-npm" in error.message
    else:
        raise AssertionError("a config whose npm half drops the crate glob must raise")


def test_declares_the_config_argument_defaulting_to_the_release_config():
    (argument,) = cli.params
    assert argument.name == "config"
    assert argument.default == RELEASE_CONFIG


def test_the_release_config_is_putitoutthere_toml():
    assert RELEASE_CONFIG == "putitoutthere.toml"
