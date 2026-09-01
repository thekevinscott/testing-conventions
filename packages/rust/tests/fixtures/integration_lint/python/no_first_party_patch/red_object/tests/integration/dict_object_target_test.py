# Red fixture for `no-first-party-patch` (#576): `patch.dict` with an object
# target. `config.registry` resolves to `myproject.config.registry` — first-party
# state, mutated for the test's benefit instead of running for real.
from unittest.mock import patch

import pytest

from myproject import config
from myproject.charge import charge


@pytest.fixture
def fast_path():
    # VIOLATION: `config.registry` is `myproject.config.registry` — first-party.
    with patch.dict(config.registry, {"fast_path": True}):
        yield


def describe_charge():
    def it_takes_the_fast_path(fast_path):
        charge(amount=100)
