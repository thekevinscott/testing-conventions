"""Run each build command through the injected `run`, then stage the binary and node `dist/` for
artifact upload."""
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
