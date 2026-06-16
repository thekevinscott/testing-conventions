# Clean fixture: the same behaviour without `monkeypatch` — patched via a
# fixture wrapping `unittest.mock.patch` (here `patch.dict` for the env).
import os

import pytest
from unittest.mock import patch


@pytest.fixture(autouse=True)
def mock_env():
    with patch.dict(os.environ, {"MYPROJECT_TOKEN": "test-token"}):
        yield


def describe_widget():
    def it_reads_the_token():
        assert os.environ["MYPROJECT_TOKEN"] == "test-token"
