# Clean fixture for Python unit isolation (#42): the canonical unit test. It imports
# only the unit under test, and mocks its collaborator by string in a fixture
# (`patch("myproject.widget.record")`) — so no first-party collaborator is imported.
from unittest.mock import patch

import pytest

from myproject.widget import build  # unit under test


@pytest.fixture(autouse=True)
def mock_record():
    with patch("myproject.widget.record") as mock:
        yield mock


def describe_build():
    def it_builds():
        assert build() is not None
