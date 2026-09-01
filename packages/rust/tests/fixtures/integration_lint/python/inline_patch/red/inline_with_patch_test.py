# Red fixture for `no-inline-patch`: a test patches inline in its body with `with patch(...)`.
# The patch belongs in a pytest.fixture, not the test body.
from unittest.mock import patch


def describe_widget():
    def it_uses_the_clock():
        with patch("myproject.widget.now", return_value=0) as clock:
            result = run()
            assert result == clock.return_value
