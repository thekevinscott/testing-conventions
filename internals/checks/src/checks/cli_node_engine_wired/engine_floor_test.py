"""Colocated unit tests for the engines.node floor parse (isolation — pure string in/out)."""
from checks.cli_node_engine_wired.engine_floor import engine_floor


def test_engine_floor_reads_a_bare_major():
    assert engine_floor(">=24") == 24


def test_engine_floor_reads_a_dotted_floor_and_surrounding_space():
    assert engine_floor(" >= 20.20.0 ") == 20


def test_engine_floor_rejects_a_requirement_it_cannot_read():
    try:
        engine_floor("^24")
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "`engines.node` is `^24`" in error.message
    else:
        raise AssertionError("an unreadable requirement must raise")
