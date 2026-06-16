# Red fixture for R2 (#50): a test patches with a bare `patch(...)` call (no `with`).
# The patch belongs in a pytest.fixture, not the test body.
from unittest.mock import patch


def describe_widget():
    def it_uses_the_clock():
        clock = patch("myproject.widget.now", return_value=0).start()
        result = run()
        assert result == clock.return_value
