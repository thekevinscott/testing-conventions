# Clean fixture for the Python integration-isolation rule (#42). An integration
# test mocks only the outside world — third-party packages and effectful stdlib —
# while first-party code (`myproject.charge` and its collaborators) runs for real.
from unittest.mock import patch

import pytest

from myproject.charge import charge


@pytest.fixture
def fake_gateway():
    # third-party package — fine to mock in an integration test
    with patch("requests.post") as mock:
        yield mock


@pytest.fixture
def fake_subprocess():
    # effectful stdlib (subprocess) — fine to mock
    with patch("subprocess.run") as mock:
        yield mock


def describe_charge():
    def it_charges_via_the_gateway(fake_gateway, fake_subprocess):
        charge(amount=100)
        assert fake_gateway.called
