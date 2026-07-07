"""End-to-end: the built wheel for this package ships no colocated `*_test.py` files.

Real `uv build` against the real `pyproject.toml`, real wheel, real zip inspection — no mocks. This
package colocates 18 `*_test.py` units beside their source (see `AGENTS.md`), so a wheel built
without an exclude rule ships them, which the packaging gate's scan rejects (#354).
"""
import shutil
import subprocess
import zipfile
from pathlib import Path

PACKAGE_ROOT = Path(__file__).resolve().parents[2]


def test_wheel_ships_no_test_files(tmp_path):
    subprocess.run(
        [shutil.which("uv"), "build", "--wheel", "-o", str(tmp_path)],
        cwd=PACKAGE_ROOT,
        check=True,
        capture_output=True,
    )
    wheel = next(tmp_path.glob("*.whl"))
    names = zipfile.ZipFile(wheel).namelist()
    test_files = [n for n in names if n.endswith("_test.py")]
    assert test_files == []
