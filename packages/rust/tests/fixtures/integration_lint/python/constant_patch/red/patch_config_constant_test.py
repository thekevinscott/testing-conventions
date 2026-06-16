# Red fixture for R4 (#52): a fixture patches a module-global config CONSTANT.
# Inject config explicitly instead of patching the constant.
import pytest
from unittest.mock import patch


@pytest.fixture
def cache_dir(tmp_path):
    with patch("myproject.config.CACHE_DIR", tmp_path):
        yield tmp_path


def describe_widget():
    def it_uses_the_cache(cache_dir):
        assert run(cache_dir) is not None
