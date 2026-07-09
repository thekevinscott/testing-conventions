# Same first-party patch as the red fixture — but this file is waived in
# `testing-conventions.toml`, so `no-first-party-patch` is lifted and the suite passes.
from unittest.mock import patch

import pytest

from myproject.charge import charge


@pytest.fixture
def fake_ledger():
    with patch("myproject.ledger.record") as mock:
        yield mock


def describe_charge():
    def it_records_the_charge(fake_ledger):
        charge(amount=100)
        assert fake_ledger.called
