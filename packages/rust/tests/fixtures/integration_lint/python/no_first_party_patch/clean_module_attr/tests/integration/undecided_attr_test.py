# Clean fixture for `no-first-party-patch`: object-form attributes the module's
# source leaves unnamed. `mystery` has no source file and `dyn.transport` is
# bound dynamically, so neither resolves to a target and both are left alone.
from unittest.mock import patch

import pytest

from myproject import dyn, mystery


@pytest.fixture
def fake_client():
    with patch.object(mystery.client, "send") as mock:
        yield mock


@pytest.fixture
def fake_transport():
    with patch.object(dyn.transport, "send") as mock:
        yield mock


def describe_sync():
    def it_sends(fake_client, fake_transport):
        assert fake_client is not fake_transport
