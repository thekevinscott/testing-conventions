# Same un-mocked first-party collaborator as the red fixture — but this file is
# waived in `testing-conventions.toml`, so `unmocked-collaborator` is lifted.
from myproject.widget import build

from myproject.ledger import record


def describe_build():
    def it_builds():
        assert build(record) is not None
