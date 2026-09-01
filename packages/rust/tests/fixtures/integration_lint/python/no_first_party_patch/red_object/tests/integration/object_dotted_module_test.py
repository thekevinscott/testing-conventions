# Red fixture for `no-first-party-patch` (#576): the object form with a dotted
# module. `myproject.ledger` names the first-party module directly.
import myproject.ledger
from unittest.mock import patch

import pytest

from myproject.charge import charge


@pytest.fixture
def fake_ledger():
    # VIOLATION: `myproject.ledger` is first-party — run it for real.
    with patch.object(myproject.ledger, "record") as mock:
        yield mock


def describe_charge():
    def it_records_the_charge(fake_ledger):
        charge(amount=100)
        assert fake_ledger.called
