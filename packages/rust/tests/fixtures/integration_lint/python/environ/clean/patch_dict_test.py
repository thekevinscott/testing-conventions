# Clean fixture for R3: env is set via patch.dict(os.environ, {...}) in a fixture.
# Reading os.environ is fine; only direct mutation is flagged.
import os

import pytest
from unittest.mock import patch


@pytest.fixture(autouse=True)
def env():
    with patch.dict(os.environ, {"MYPROJECT_TOKEN": "test-token"}):
        yield


def describe_widget():
    def it_reads_the_token():
        assert os.environ["MYPROJECT_TOKEN"] == "test-token"
