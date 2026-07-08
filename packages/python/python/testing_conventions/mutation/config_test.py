"""Unit tests for the mutation adapter's cosmic-ray config."""
from testing_conventions.mutation.config import EXCLUDES, build_config, render_config


def test_renders_the_whole_project_when_no_modules():
    toml = render_config([], 12.0)
    assert 'module-path = ["."]' in toml
    assert 'name = "local"' in toml
    assert "python3 -m pytest" in toml
    for glob in EXCLUDES:
        assert f'"{glob}"' in toml


def test_renders_the_given_modules_scoped():
    toml = render_config(["a.py", "pkg/b.py"], 12.0)
    assert 'module-path = ["a.py", "pkg/b.py"]' in toml


def test_renders_the_passed_timeout_rather_than_a_fixed_value():
    # The per-run timeout is passed in (derived from the observed suite runtime), not hardcoded
    # at 30s — so a suite slower than 30s no longer times out on a fixed ceiling.
    assert "timeout = 42.0\n" in render_config([], 42.0)
    assert "timeout = 30.0\n" not in render_config([], 7.5)


def test_build_config_parses_the_rendered_toml(cosmic_ray):
    cosmic_ray.deserialize_config.return_value = {"cfg": True}
    assert build_config(["a.py"], 12.0) == {"cfg": True}
    cosmic_ray.deserialize_config.assert_called_once_with(render_config(["a.py"], 12.0))


def test_the_per_mutant_suite_stops_at_the_first_failure():
    # -x: a killed mutant's run ends at the test that kills it. A survivor's all-green run never
    # triggers it and still exits 0, and an interrupted run still exits non-zero, so cosmic-ray's
    # exit-status classification — hence the survivor set — is unchanged.
    assert (
        'test-command = "python3 -m pytest -x -q -p no:cacheprovider"\n'
        in render_config([], 12.0)
    )


def test_derive_timeout_scales_with_the_observed_runtime():
    # A slow suite gets a proportionally larger budget (observed x multiplier)...
    from testing_conventions.mutation.config import derive_timeout

    assert derive_timeout(20.0) == 60.0


def test_derive_timeout_floors_a_fast_suite():
    # ...but a sub-second suite still gets the floor (10s), not a razor-thin budget. Pinned to
    # the literal, not to ``MIN_TIMEOUT`` itself, so shrinking the floor is caught.
    from testing_conventions.mutation.config import MIN_TIMEOUT, derive_timeout

    assert derive_timeout(0.1) == 10.0 == MIN_TIMEOUT
