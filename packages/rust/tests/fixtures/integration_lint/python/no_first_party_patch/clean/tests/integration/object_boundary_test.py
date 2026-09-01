# Clean fixture for `no-first-party-patch`: the object form at the system
# boundary. `requests` is third-party, and a base bound by no import (a call
# result, a fixture argument) resolves to no module and is left alone.
from unittest.mock import patch

import pytest

import requests

from myproject.charge import charge


@pytest.fixture
def fake_gateway():
    # third-party package — fine to mock in an integration test
    with patch.object(requests, "post") as mock:
        yield mock


@pytest.fixture
def fake_transport():
    # a call result — statically unresolvable, so the rule leaves it alone
    with patch.object(build_transport(), "send") as mock:
        yield mock


@pytest.fixture
def fake_client(client):
    # a fixture argument — bound by no import, left alone
    with patch.object(client, "send") as mock:
        yield mock


def describe_charge():
    def it_charges_via_the_gateway(fake_gateway):
        charge(amount=100)
        assert fake_gateway.called
