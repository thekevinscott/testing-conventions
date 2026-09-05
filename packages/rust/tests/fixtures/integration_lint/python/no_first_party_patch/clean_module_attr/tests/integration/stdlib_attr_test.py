# Clean fixture for `no-first-party-patch`: the object form reaching stdlib
# through a first-party module. `async_mod.asyncio` is the stdlib `asyncio`
# module — an effectful boundary, fine to patch in an integration test.
from unittest.mock import patch

import pytest

from myproject import async_mod


@pytest.fixture
def spy_to_thread():
    with patch.object(async_mod.asyncio, "to_thread") as spy:
        yield spy


@pytest.fixture
def fake_cwd():
    with patch.object(async_mod.os, "getcwd", return_value="/tmp") as mock:
        yield mock


def describe_run_query():
    def it_offloads_to_a_thread(spy_to_thread, fake_cwd):
        async_mod.run_query("select 1")
        assert spy_to_thread.called
