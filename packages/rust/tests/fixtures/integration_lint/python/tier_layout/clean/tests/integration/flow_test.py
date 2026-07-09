# Clean: the third-party boundary is patched via a fixture wrapping
# `unittest.mock.patch`; first-party code runs for real.
import os

import pytest
from unittest.mock import patch


@pytest.fixture(autouse=True)
def mock_env():
    with patch.dict(os.environ, {"MYPROJECT_TOKEN": "test-token"}):
        yield


def describe_flow():
    def it_reads_the_token():
        assert os.environ["MYPROJECT_TOKEN"] == "test-token"
