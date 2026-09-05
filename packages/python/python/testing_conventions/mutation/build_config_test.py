"""Unit tests for the config parse: the rendered TOML reaches cosmic-ray unchanged."""
from unittest import mock

import pytest

import testing_conventions.mutation.build_config as unit


@pytest.fixture
def mock_render_config():
    with mock.patch.object(unit, "render_config", return_value="[cosmic-ray]\n") as patched:
        yield patched


def test_build_config_parses_the_rendered_toml(cosmic_ray, mock_render_config):
    cosmic_ray.deserialize_config.return_value = {"cfg": True}
    assert unit.build_config(["a.py"], 12.0) == {"cfg": True}
    mock_render_config.assert_called_once_with(["a.py"], 12.0)
    cosmic_ray.deserialize_config.assert_called_once_with("[cosmic-ray]\n")
