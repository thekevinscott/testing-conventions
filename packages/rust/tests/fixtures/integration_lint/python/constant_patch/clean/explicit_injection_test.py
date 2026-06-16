# Clean fixture for R4: config is injected explicitly, not patched.
import pytest


@pytest.fixture
def cache_dir(tmp_path):
    return tmp_path


def describe_widget():
    def it_uses_the_cache(cache_dir):
        assert run(cache_dir) is not None
