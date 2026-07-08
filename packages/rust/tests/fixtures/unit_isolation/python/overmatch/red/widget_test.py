# Red fixture for Python unit isolation (#393, defect 1a — any-symbol-clears-all).
# A `from … import a, b` collaborator is mocked only when EVERY imported symbol is
# individually patched. Here the fixture patches `myproject.ledger.record` but leaves
# the sibling `erase` un-mocked — so the import still reaches a real collaborator and
# must be flagged.
import pytest

from myproject.widget import build  # unit under test — never a collaborator

from myproject.ledger import record, erase  # `erase` is NOT mocked → violation


@pytest.fixture(autouse=True)
def mock_record(mocker):
    mocker.patch("myproject.ledger.record")


def describe_build():
    def it_builds():
        # Uses the real `erase` collaborator — not isolated.
        assert build(record, erase) is not None
