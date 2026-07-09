# An integration test: first-party code runs for real, deliberately unmocked.
# The unit-suite isolation rule holds no claim under `<package root>/tests/`.
from myproject.widget import build

from myproject.ledger import record


def describe_flow():
    def it_builds():
        assert build(record) is not None
