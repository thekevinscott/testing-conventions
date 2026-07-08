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

# The per-mutant timeout is scoped to the clean suite's observed runtime rather than a fixed
# ceiling: a mutant whose run outlasts the clean suite by more than ``TIMEOUT_MULTIPLIER`` is
# judged hung (an inconclusive timeout), while a suite that is merely slow earns a
# proportionally larger budget. A fixed 30s instead timed out any suite slower than 30s — and,
# before the baseline guard tightened, false-greened it (#395).
TIMEOUT_MULTIPLIER = 3.0
# A floor so a sub-second suite still gets a usable budget across process startup and jitter.
MIN_TIMEOUT = 10.0
# The generous ceiling the clean suite is *measured* under (the baseline run). A slow-but-
# working suite completes within it and is measured; a hung suite hits it and fails the
# baseline guard loudly rather than measuring forever. This bounds only the measurement — the
# per-mutant budget is derived from the observed runtime, not from this value.
MEASURE_TIMEOUT = 300.0


def derive_timeout(observed_seconds):
    """The per-mutant timeout derived from the clean suite's observed wall-clock
    ``observed_seconds``: ``observed x TIMEOUT_MULTIPLIER``, floored at ``MIN_TIMEOUT``. Pure
    over its input."""
    return max(MIN_TIMEOUT, observed_seconds * TIMEOUT_MULTIPLIER)


def render_config(modules, timeout):
    """Render the ``cosmic-ray`` TOML for a run over ``modules`` (a list of source-file
    paths; empty ⇒ the whole project, ``"."``) with the per-run ``timeout`` in seconds.
    ``pytest`` runs the suite, ending a killed mutant's run at its first failing test (``-x``;
    exit status — hence classification — is unchanged); the ``local`` distributor runs each
    mutant in this adapter's own process tree."""
    paths = modules or ["."]
    module_path = ", ".join(f'"{p}"' for p in paths)
    excludes = ", ".join(f'"{glob}"' for glob in EXCLUDES)
    return (
        "[cosmic-ray]\n"
        f"module-path = [{module_path}]\n"
        f"timeout = {timeout}\n"
        f"excluded-modules = [{excludes}]\n"
        'test-command = "python3 -m pytest -x -q -p no:cacheprovider"\n'
        "\n"
        "[cosmic-ray.distributor]\n"
        'name = "local"\n'
    )


def build_config(modules, timeout):
    """The parsed cosmic-ray ``ConfigDict`` for a run over ``modules`` with ``timeout``."""
    from cosmic_ray.config import deserialize_config

    return deserialize_config(render_config(modules, timeout))
