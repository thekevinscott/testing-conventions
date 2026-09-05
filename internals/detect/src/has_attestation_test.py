import pytest

import has_attestation


def test_has_attestation_accepts_the_legacy_single_receipt(tmp_path, write):
    write(tmp_path / "e2e-attestation.json", "{}")
    assert has_attestation.has_attestation(tmp_path) is True


def test_has_attestation_accepts_a_branch_keyed_receipt(tmp_path, write):
    write(tmp_path / "e2e-attestations" / "main.json", "{}")
    assert has_attestation.has_attestation(tmp_path) is True


@pytest.mark.parametrize("entry", ["README.md", "main.cfg"])
def test_has_attestation_ignores_a_non_json_entry(tmp_path, write, entry):
    write(tmp_path / "e2e-attestations" / entry)
    assert has_attestation.has_attestation(tmp_path) is False


def test_has_attestation_ignores_a_directory_named_like_a_receipt(tmp_path):
    (tmp_path / "e2e-attestations" / "main.json").mkdir(parents=True)
    assert has_attestation.has_attestation(tmp_path) is False


def test_has_attestation_is_false_with_no_receipts(tmp_path):
    assert has_attestation.has_attestation(tmp_path) is False
