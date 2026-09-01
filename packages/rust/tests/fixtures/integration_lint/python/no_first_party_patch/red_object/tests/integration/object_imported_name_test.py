# Red fixture for `no-first-party-patch` (#576): the object form with an imported
# name. `ledger` resolves through `from myproject import ledger`, so the patch
# targets first-party code exactly like `patch("myproject.ledger.record")`.
from unittest.mock import patch

import pytest

from myproject import ledger
from myproject.charge import charge


@pytest.fixture
def fake_ledger():
    # VIOLATION: `ledger` is `myproject.ledger` — first-party, run it for real.
    with patch.object(ledger, "record") as mock:
        yield mock


def describe_charge():
    def it_records_the_charge(fake_ledger):
        charge(amount=100)
        assert fake_ledger.called
