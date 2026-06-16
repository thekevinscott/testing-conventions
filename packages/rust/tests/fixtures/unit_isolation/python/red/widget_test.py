# Red fixture for Python unit isolation (#42, `unmocked-collaborator`). A unit test
# must isolate the unit under test, so importing a first-party collaborator without
# mocking it is the violation — patch it (by string, in a fixture) instead.
from myproject.widget import build  # unit under test — never a collaborator

from myproject.ledger import record  # first-party collaborator, NOT mocked → violation


def describe_build():
    def it_builds():
        # Uses the real `record` collaborator — not isolated.
        assert build(record) is not None
