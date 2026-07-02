"""Build the cosmic-ray configuration the adapter drives.

[`render_config`] is the pure TOML-rendering half; [`build_config`] parses it into
cosmic-ray's ``ConfigDict`` (the object ``init`` / ``execute`` consume). ``cosmic_ray`` is
imported lazily so the module imports without the engine installed — the pure renderer stays
testable anywhere, and the dogfood coverage job (which installs only coverage + pytest) can
collect this package.
"""
from __future__ import annotations

# Test files cosmic-ray must never mutate (it would mutate the suite itself). Mirrors the
# excludes the CLI-driven Rust arm used.
EXCLUDES = ["*_test.py", "test_*.py", "conftest.py"]


def render_config(modules):
    """Render the ``cosmic-ray`` TOML for a run over ``modules`` (a list of source-file
    paths; empty ⇒ the whole project, ``"."``). ``pytest`` runs the suite; the ``local``
    distributor runs each mutant in this adapter's own process tree."""
    paths = modules or ["."]
    module_path = ", ".join(f'"{p}"' for p in paths)
    excludes = ", ".join(f'"{glob}"' for glob in EXCLUDES)
    return (
        "[cosmic-ray]\n"
        f"module-path = [{module_path}]\n"
        "timeout = 30.0\n"
        f"excluded-modules = [{excludes}]\n"
        'test-command = "python3 -m pytest -q -p no:cacheprovider"\n'
        "\n"
        "[cosmic-ray.distributor]\n"
        'name = "local"\n'
    )


def build_config(modules):
    """The parsed cosmic-ray ``ConfigDict`` for a run over ``modules``."""
    from cosmic_ray.config import deserialize_config

    return deserialize_config(render_config(modules))
