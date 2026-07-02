"""Run a cosmic-ray session over the configured modules.

``cosmic_ray`` is imported lazily (the module imports without the engine; the session is
unit-tested with a fake ``cosmic_ray`` injected).
"""
from __future__ import annotations


def run_session(config):
    """Init + execute cosmic-ray over ``config``'s ``module-path`` and return a list of
    ``(MutationSpec, WorkResult)`` — the first mutation of each completed work item paired
    with its outcome. The session DB lives in an out-of-tree temp dir; cosmic-ray mutates
    each source file in place and reverts it, so the scanned tree is left as it was."""
    import tempfile
    from pathlib import Path

    import cosmic_ray.commands
    import cosmic_ray.modules
    from cosmic_ray.work_db import WorkDB, use_db

    module_path = config["module-path"]
    paths = (
        [Path(module_path)]
        if isinstance(module_path, str)
        else [Path(entry) for entry in module_path]
    )
    modules = cosmic_ray.modules.find_modules(paths)
    modules = cosmic_ray.modules.filter_paths(modules, config.get("excluded-modules", ()))
    with tempfile.TemporaryDirectory() as tmp:
        with use_db(Path(tmp) / "session.sqlite", mode=WorkDB.Mode.create) as database:
            cosmic_ray.commands.init(modules, database, config.operators_config)
            cosmic_ray.commands.execute(database, config)
            return [
                (item.mutations[0], result) for item, result in database.completed_work_items
            ]
