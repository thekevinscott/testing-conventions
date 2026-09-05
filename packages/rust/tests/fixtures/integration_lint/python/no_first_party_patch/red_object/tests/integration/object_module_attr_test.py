# Red fixture for `no-first-party-patch`: the object form reaching a first-party
# module attribute. `async_mod.py` holds `from . import helper`, so
# `async_mod.helper` is `myproject.helper` — first-party, run it for real.
from unittest.mock import patch

import pytest

from myproject import async_mod


@pytest.fixture
def fake_helper():
    # VIOLATION: `async_mod.helper` is `myproject.helper` — first-party.
    with patch.object(async_mod.helper, "run") as mock:
        yield mock


def describe_client():
    def it_runs_the_helper(fake_helper):
        async_mod.Client().send("payload")
        assert fake_helper.called
