# Red fixture for the Python integration-isolation rule (#42, `no-first-party-patch`).
# An integration test runs first-party code for real, so patching a first-party
# target is the violation. The patch lives in a fixture (so it does NOT also trip
# `no-inline-patch`) — isolating the new rule.
from unittest.mock import patch

import pytest

from myproject.charge import charge


@pytest.fixture
def fake_ledger():
    # VIOLATION: `myproject.ledger` is first-party — an integration test must run
    # it for real. Mock the outside world, not our own code.
    with patch("myproject.ledger.record") as mock:
        yield mock


def describe_charge():
    def it_records_the_charge(fake_ledger):
        charge(amount=100)
        assert fake_ledger.called
