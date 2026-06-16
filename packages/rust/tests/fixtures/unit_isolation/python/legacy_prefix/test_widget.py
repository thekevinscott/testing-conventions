# Red fixture for #145: a *legacy* `test_*.py` importing an un-mocked first-party
# collaborator (`unmocked-collaborator`). After #112 a unit test is `*_test.py` and a
# `test_*.py` is ordinary *source*, so `unit isolation` must NOT scan this file — even
# though the same body in a `*_test.py` would be flagged (see the sibling `red`).
from myproject.widget import build  # unit under test — never a collaborator

from myproject.ledger import record  # first-party collaborator, NOT mocked


def describe_build():
    def it_builds():
        assert build(record) is not None
