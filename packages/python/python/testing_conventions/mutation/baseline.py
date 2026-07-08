"""The baseline check: the unit suite must pass over unmutated code.

``cosmic_ray`` is imported lazily (the module imports without the engine; the check itself
is unit-tested with a fake ``cosmic_ray`` injected).
"""
from __future__ import annotations

import time


def check_baseline(config, clock=time.monotonic):
    """Run the suite once with no mutation and return the clean run's observed wall-clock
    seconds (the per-mutant timeout is scoped to it), measured with ``clock`` (injected so a
    test drives the timing deterministically). Raise ``RuntimeError`` unless the run *passes* —
    cosmic-ray reports the unmutated baseline ``survived`` when the suite passes on its own. Any
    other outcome is a loud failure: ``killed`` (the suite failed unmutated, so every mutant
    would falsely ``die`` on the already-red suite and a clean pass be reported), or a timeout /
    abnormal / no-test run (``test_outcome`` is ``None`` or otherwise not ``survived``) — a suite
    too slow for its budget would otherwise time out silently, every mutant then time out and
    drop, and an empty survivor set slip through as a false green."""
    import tempfile
    from pathlib import Path

    from cosmic_ray.commands import execute
    from cosmic_ray.work_db import WorkDB, use_db
    from cosmic_ray.work_item import WorkItem

    with tempfile.TemporaryDirectory() as tmp:
        with use_db(Path(tmp) / "baseline.sqlite", mode=WorkDB.Mode.create) as database:
            database.clear()
            database.add_work_item(WorkItem(mutations=[], job_id="baseline"))
            started = clock()
            execute(database, config)
            elapsed = clock() - started
            _, result = next(database.results)
    outcome = getattr(result.test_outcome, "value", result.test_outcome)
    if outcome != "survived":
        raise RuntimeError(
            "the Python unit suite did not pass unmutated (cosmic-ray baseline outcome was "
            f"{outcome!r}, not 'survived'):\n" + (result.output or "")
        )
    return elapsed
