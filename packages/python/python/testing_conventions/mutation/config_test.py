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


def test_the_per_mutant_suite_stops_at_the_first_failure():
    # -x (#380): a killed mutant's run ends at the test that kills it. A survivor's all-green
    # run never triggers it and still exits 0, and an interrupted run still exits non-zero, so
    # cosmic-ray's exit-status classification — hence the survivor set — is unchanged.
    assert 'test-command = "python3 -m pytest -x -q -p no:cacheprovider"\n' in render_config([])
