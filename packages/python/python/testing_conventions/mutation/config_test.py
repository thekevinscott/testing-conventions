"""Unit tests for the mutation adapter's cosmic-ray config (#248)."""
from testing_conventions.mutation.config import EXCLUDES, build_config, render_config


def test_renders_the_whole_project_when_no_modules():
    toml = render_config([])
    assert 'module-path = ["."]' in toml
    assert 'name = "local"' in toml
    assert "python3 -m pytest" in toml
    for glob in EXCLUDES:
        assert f'"{glob}"' in toml


def test_renders_the_given_modules_scoped():
    toml = render_config(["a.py", "pkg/b.py"])
    assert 'module-path = ["a.py", "pkg/b.py"]' in toml


def test_build_config_parses_the_rendered_toml(cosmic_ray):
    cosmic_ray.deserialize_config.return_value = {"cfg": True}
    assert build_config(["a.py"]) == {"cfg": True}
    cosmic_ray.deserialize_config.assert_called_once_with(render_config(["a.py"]))
