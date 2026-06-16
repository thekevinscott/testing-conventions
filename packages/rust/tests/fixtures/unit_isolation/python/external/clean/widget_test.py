# Clean fixture for slice 3: the canonical unit test mocks external collaborators by
# string (so they're never imported), and may use pure stdlib freely.
from unittest.mock import patch

import pytest

from myproject.widget import build  # unit under test
import json  # pure stdlib — fine


@pytest.fixture(autouse=True)
def mock_http():
    with patch("myproject.widget.requests") as mock:
        yield mock


@pytest.fixture(autouse=True)
def mock_run():
    with patch("myproject.widget.subprocess") as mock:
        yield mock


def describe_build():
    def it_builds():
        assert build(json) is not None
