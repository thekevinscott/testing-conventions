# A legacy `test_*.py` name. Under the greenfield convention (#112) this is
# ordinary source, not a colocated unit test, so `unit isolation` never scans it —
# even though it imports a first-party collaborator (`myproject.ledger`) without
# mocking it, which would be an `unmocked-collaborator` violation in a real unit
# test (`*_test.py`).
from myproject.widget import build  # unit under test — never a collaborator

from myproject.ledger import record  # first-party collaborator, NOT mocked


def describe_build():
    def it_builds():
        assert build(record) is not None
