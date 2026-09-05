# Red fixture for `no-first-party-patch`: the object form on a class the module
# defines. `async_mod.py` defines `Client`, so `async_mod.Client` is
# first-party — run it for real.
from unittest.mock import patch

import pytest

from myproject import async_mod


@pytest.fixture
def fake_send():
    # VIOLATION: `async_mod.Client` is defined in `myproject.async_mod` — first-party.
    with patch.object(async_mod.Client, "send") as mock:
        yield mock


def describe_client():
    def it_sends(fake_send):
        async_mod.Client().send("payload")
        assert fake_send.called
