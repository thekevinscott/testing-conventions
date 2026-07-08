"""Build and stage the hermetic CLI artifact — the shared orchestrator for `build-hermetic-cli`
(#356, epic #353).

`stage_hermetic_cli(commands, binary, node_dist, stage_dir, root=".", run=subprocess.run)` runs
each `(argv, cwd)` in `commands` through `run`, raising `CheckFailed` on the first non-zero exit
(stages nothing), then copies `binary` to `<stage_dir>/testing-conventions` (exec bit set —
artifact upload/download preserves paths, not modes, so the workflow's download step re-chmods on
the other side) and `node_dist` to `<stage_dir>/dist`.

The subprocess boundary is injected as `run` (defaulting to `subprocess.run`), the `run_checks`
pattern (#328); `root` is injected the same way. Both keep the seams here, on this shared
orchestrator, rather than smuggled as test-only parameters onto `build_hermetic_cli.cli`'s public
click signature (click would never bind either one there).
"""
from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

from checks.utils.check_failed import CheckFailed


def stage_hermetic_cli(commands, binary, node_dist, stage_dir, root=".", run=subprocess.run) -> None:
    root_path = Path(root)
    for argv, cwd in commands:
        result = run(argv, cwd=str(root_path / cwd))
        if result.returncode != 0:
            raise CheckFailed(f"`{' '.join(argv)}` exited {result.returncode}")
    stage = Path(stage_dir)
    stage.mkdir(parents=True, exist_ok=True)
    staged_binary = stage / "testing-conventions"
    shutil.copyfile(root_path / binary, staged_binary)
    staged_binary.chmod(0o755)
    shutil.copytree(root_path / node_dist, stage / "dist", dirs_exist_ok=True)
