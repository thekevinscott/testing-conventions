"""Colocated unit tests for the action-manifest `outputs:` parse (isolation — a text argument)."""
from checks.published_detect_contract.declared_outputs import declared_outputs

MANIFEST = """name: 'Detect languages'

inputs:
  languages:
    default: ''

outputs:
  package_root:
    description: 'The derived package root.'
    value: ${{ steps.scan.outputs.package_root }}
  e2e_attestation:
    description: 'true when receipts sit at the package root.'
    value: ${{ steps.scan.outputs.e2e_attestation }}

runs:
  using: 'composite'
  steps:
    - id: scan
      shell: bash
      run: python3 detect.py
"""


def test_the_declared_names_are_the_two_space_keys_of_the_outputs_mapping():
    assert declared_outputs(MANIFEST) == {"package_root", "e2e_attestation"}


def test_an_output_name_carrying_a_digit_survives_the_parse():
    # `[a-z_]+` would drop `e2e_attestation` from the declared side, and the superset the check
    # asserts would hold over a set missing the very names most likely to be new.
    assert "e2e_attestation" in declared_outputs(MANIFEST)


def test_the_inputs_mapping_is_not_read_as_an_output():
    assert "languages" not in declared_outputs(MANIFEST)


def test_the_runs_mapping_ends_the_block():
    assert declared_outputs(MANIFEST).isdisjoint({"using", "steps"})


def test_an_outputs_mapping_that_ends_the_file_is_read_to_its_last_name():
    trailing = MANIFEST.partition("\nruns:\n")[0] + "\n"
    assert declared_outputs(trailing) == {"package_root", "e2e_attestation"}


def test_a_manifest_declaring_no_outputs_declares_the_empty_set():
    assert declared_outputs("name: 'Detect languages'\nruns:\n  using: 'composite'\n") == set()
