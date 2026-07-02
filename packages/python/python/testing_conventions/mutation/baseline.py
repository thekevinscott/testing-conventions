"""The baseline check: the unit suite must pass over unmutated code.

``cosmic_ray`` is imported lazily (the module imports without the engine; the check itself
is unit-tested with a fake ``cosmic_ray`` injected).
"""
from __future__ import annotations


def check_baseline(config):
    """Run the suite once with no mutation, raising ``RuntimeError`` if it fails. cosmic-ray
    reports the unmutated baseline ``killed`` when the tests fail on their own — without this
    guard every mutant would falsely ``die`` on the already-failing suite and we'd report a
    clean pass."""
    import tempfile
    from pathlib import Path

    from cosmic_ray.commands import execute
    from cosmic_ray.work_db import WorkDB, use_db
    from cosmic_ray.work_item import WorkItem

    with tempfile.TemporaryDirectory() as tmp:
        with use_db(Path(tmp) / "baseline.sqlite", mode=WorkDB.Mode.create) as database:
            database.clear()
            database.add_work_item(WorkItem(mutations=[], job_id="baseline"))
            execute(database, config)
            _, result = next(database.results)
    outcome = getattr(result.test_outcome, "value", result.test_outcome)
    if outcome == "killed":
        raise RuntimeError(
            "the Python unit suite did not pass unmutated (cosmic-ray baseline failed):\n"
            + (result.output or "")
        )
