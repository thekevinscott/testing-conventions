# Integration test that trips `no-constant-patch` by patching `widget.MAX_RETRIES`. The patch
# sits in a fixture so `no-inline-patch` stays quiet, leaving the constant patch as the one
# violation — the one a `[[python.exempt]]` entry in testing-conventions.toml waives.
from unittest.mock import patch

import pytest

import widget


@pytest.fixture
def few_retries():
    with patch("widget.MAX_RETRIES", 1):
        yield


def test_retries_are_capped(few_retries):
    assert widget.MAX_RETRIES == 1
