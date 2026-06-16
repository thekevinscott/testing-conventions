# Waived fixture for R2 (#50): identical to the red fixture (an inline
# `with patch(...)` in a test body), but waived in the colocated
# testing-conventions.toml — so it must be silent.
from unittest.mock import patch


def describe_widget():
    def it_uses_the_clock():
        with patch("myproject.widget.now", return_value=0) as clock:
            result = run()
            assert result == clock.return_value
