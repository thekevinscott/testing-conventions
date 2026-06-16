# Waived fixture for R4: identical to the red fixture (patches a config CONSTANT),
# but waived in the colocated testing-conventions.toml — so it must be silent.
import pytest
from unittest.mock import patch


@pytest.fixture
def cache_dir(tmp_path):
    with patch("myproject.config.CACHE_DIR", tmp_path):
        yield tmp_path


def describe_widget():
    def it_uses_the_cache(cache_dir):
        assert run(cache_dir) is not None
