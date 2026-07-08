# Red fixture for Python unit isolation (#393, defect 1b — last-segment match against
# any target). A patch mocks an imported symbol only when the patch target's MODULE
# PATH corresponds to the import's source, not merely its last dotted segment. Here
# `patch("otherpkg.unrelated.record")` shares a last segment with `record` but names a
# different module, and `patch("json.dumps")` shares a last segment with `dumps` but is
# stdlib — neither mocks the local collaborator, so both imports are flagged.
import pytest

from myproject.widget import build  # unit under test — never a collaborator

from myproject.ledger import record  # NOT mocked: the patch below names another module
from myproject.formatter import dumps  # NOT mocked: `json.dumps` is a different module


@pytest.fixture(autouse=True)
def mocks(mocker):
    mocker.patch("otherpkg.unrelated.record")
    mocker.patch("json.dumps")


def describe_build():
    def it_builds():
        # Uses the real `record` and `dumps` collaborators — not isolated.
        assert build(record, dumps) is not None
