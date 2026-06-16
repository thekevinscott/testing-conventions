# Integration test that trips `no-constant-patch` (#52): it patches a module-global
# UPPER_CASE constant (`widget.MAX_RETRIES`). The patch lives in a fixture (so
# `no-inline-patch` does not fire) and first-party code runs for real — the one
# remaining violation is the constant patch, which a `[[python.exempt]]` entry in
# testing-conventions.toml waives (#102). The reusable workflow's integration-lint
# job only honors that waiver if it forwards `--config` (#126).
from unittest.mock import patch

import pytest

import widget


@pytest.fixture
def few_retries():
    with patch("widget.MAX_RETRIES", 1):
        yield


def test_retries_are_capped(few_retries):
    assert widget.MAX_RETRIES == 1
