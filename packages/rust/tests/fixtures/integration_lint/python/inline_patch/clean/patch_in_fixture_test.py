# Clean fixture for R2: the patch lives in a pytest.fixture, not the test body.
from unittest.mock import patch

import pytest


@pytest.fixture
def clock():
    with patch("myproject.widget.now", return_value=0) as mock:
        yield mock


def describe_widget():
    def it_uses_the_clock(clock):
        result = run()
        assert result == clock.return_value
