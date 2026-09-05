"""Colocated unit tests for the dispatch-and-wait command (isolation — no `CliRunner`, no git/gh).

The dispatch/poll mechanics are covered beside `checks/utils/verify_release/`; here the callback's
raise-or-echo branch is driven through `.callback()` with the imported collaborators patched.
"""
from checks.verify_release.dispatch_and_wait import dispatch_and_wait


def _record(seen, name, answer):
    def call(*args):
        seen[name] = args
        return answer

    return call


def test_dispatch_and_wait_takes_sha_version_and_variadic_workflows():
    sha, version, workflows = dispatch_and_wait.params
    assert sha.name == "sha"
    assert version.name == "version"
    assert workflows.name == "workflows"
    assert workflows.nargs == -1


def test_dispatch_and_wait_echoes_the_ok_message_when_every_suite_passed(monkeypatch, capsys):
    conclusions = {"dogfood.yml": "success"}
    seen = {}
    monkeypatch.setattr("checks.verify_release.dispatch_and_wait.verify_suites", _record(seen, "suites", conclusions))
    monkeypatch.setattr("checks.verify_release.dispatch_and_wait.failed_suites", _record(seen, "failed", []))
    monkeypatch.setattr(
        "checks.verify_release.dispatch_and_wait.verification_ok",
        _record(seen, "ok", "both suites passed at thesha"),
    )
    monkeypatch.setattr(
        "checks.verify_release.dispatch_and_wait.verification_error", _record(seen, "error", "unreachable")
    )
    dispatch_and_wait.callback(sha="thesha", version="0.0.67", workflows=("selftest.yml", "dogfood.yml"))
    assert seen["suites"] == ("thesha", "0.0.67", ["selftest.yml", "dogfood.yml"])
    assert seen["failed"] == (conclusions,)
    assert seen["ok"] == ("thesha", conclusions)
    assert capsys.readouterr().out == "both suites passed at thesha\n"


def test_dispatch_and_wait_raises_the_error_message_when_a_suite_failed(monkeypatch):
    seen = {}
    monkeypatch.setattr(
        "checks.verify_release.dispatch_and_wait.verify_suites",
        _record(seen, "suites", {"dogfood.yml": "failure"}),
    )
    monkeypatch.setattr(
        "checks.verify_release.dispatch_and_wait.failed_suites", _record(seen, "failed", ["dogfood.yml"])
    )
    monkeypatch.setattr("checks.verify_release.dispatch_and_wait.verification_ok", _record(seen, "ok", "unreachable"))
    monkeypatch.setattr(
        "checks.verify_release.dispatch_and_wait.verification_error",
        _record(seen, "error", "dogfood.yml failed at thesha"),
    )
    try:
        dispatch_and_wait.callback(sha="thesha", version="0.0.67", workflows=("dogfood.yml",))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert error.message == "dogfood.yml failed at thesha"
    else:
        raise AssertionError("a failed suite must raise")
    assert seen["error"] == ("thesha", ["dogfood.yml"])
