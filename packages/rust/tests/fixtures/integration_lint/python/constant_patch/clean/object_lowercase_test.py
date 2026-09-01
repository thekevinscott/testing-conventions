# Clean fixture for `no-constant-patch` (#576): the object form patching a
# lower-case attribute is an ordinary patch, not a config constant.
import pytest
from unittest.mock import patch

from myproject import gateway


@pytest.fixture
def fake_send():
    with patch.object(gateway, "send") as mock:
        yield mock


def describe_widget():
    def it_sends(fake_send):
        assert run() is not None
