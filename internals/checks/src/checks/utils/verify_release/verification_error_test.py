"""Colocated unit tests for the verification messages (isolation — pure strings in/out)."""
from checks.utils.verify_release.verification_error import verification_error, verification_ok


def test_verification_error_names_the_failed_suites_and_fails_closed():
    message = verification_error("thesha", ["selftest.yml (failure)"])
    assert "selftest.yml (failure)" in message
    assert "thesha" in message
    assert "refusing to promote" in message


def test_verification_ok_names_the_verified_workflows():
    assert verification_ok("thesha", ["a.yml", "b.yml"]) == \
        "the version-pinned verification passed for a.yml, b.yml at thesha"
