# Red fixture for `no-constant-patch`: the object form. `cfg` resolves
# through the import, so patching its UPPER_CASE attribute is the same
# config-constant patch as `patch("myproject.cfg.CACHE_DIR", ...)`.
import pytest
from unittest.mock import patch

from myproject import cfg


@pytest.fixture
def cache_dir(tmp_path):
    with patch.object(cfg, "CACHE_DIR", tmp_path):
        yield tmp_path


def describe_widget():
    def it_uses_the_cache(cache_dir):
        assert run(cache_dir) is not None
