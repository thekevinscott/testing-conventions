import pytest

import ts_pnpm_version


def test_pnpm_version_pin_echoes_a_packagemanager_pin():
    assert ts_pnpm_version._pnpm_version_pin("pnpm@10.33.0") == "10.33.0"


def test_pnpm_version_pin_echoes_a_pin_that_already_satisfies_the_floor():
    assert ts_pnpm_version._pnpm_version_pin("pnpm@11.11.0") == "11.11.0"


def test_pnpm_version_pin_keeps_build_metadata_in_an_echoed_pin():
    pin = "11.11.0+sha512.abc123"
    assert ts_pnpm_version._pnpm_version_pin(f"pnpm@{pin}") == pin


def test_pnpm_version_pin_is_never_empty_for_a_pnpm_pin():
    assert ts_pnpm_version._pnpm_version_pin("pnpm@10.33.0") != ""


def test_pnpm_version_pin_falls_back_to_the_floor_with_no_field():
    assert ts_pnpm_version._pnpm_version_pin("") == ts_pnpm_version.PNPM_FLOOR


@pytest.mark.parametrize("declared", ["npm@10.0.0", "yarn@4.0.0"])
def test_pnpm_version_pin_falls_back_to_the_floor_for_another_manager(declared):
    assert ts_pnpm_version._pnpm_version_pin(declared) == ts_pnpm_version.PNPM_FLOOR


def test_ts_pnpm_version_echoes_the_manifest_pin(tmp_path, write):
    write(tmp_path / "package.json", '{"packageManager": "pnpm@10.4.1"}')
    assert ts_pnpm_version.ts_pnpm_version(tmp_path) == "10.4.1"


def test_ts_pnpm_version_falls_back_to_the_floor_without_a_pin(tmp_path):
    assert ts_pnpm_version.ts_pnpm_version(tmp_path) == ts_pnpm_version.PNPM_FLOOR
