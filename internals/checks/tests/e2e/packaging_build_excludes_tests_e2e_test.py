"""End-to-end: the built distributions for this package ship no colocated `*_test.py` files.

Real `uv build` against the real `pyproject.toml`, real archive inspection — no mocks. This package
colocates 18 `*_test.py` units beside their source (see `AGENTS.md`), so a distribution built
without an exclude rule ships them, which the packaging gate's scan rejects (#354). `uv build`
produces both a wheel and an sdist, and hatchling's wheel/sdist targets exclude independently — an
exclude scoped to only one target still leaves the other shipping every test file, exactly what
happened when `[tool.hatch.build.targets.wheel]`'s `exclude` alone left the sdist unfixed.
"""
import shutil
import subprocess
import tarfile
import zipfile
from pathlib import Path

PACKAGE_ROOT = Path(__file__).resolve().parents[2]


def _build(tmp_path):
    subprocess.run(
        [shutil.which("uv"), "build", "-o", str(tmp_path)],
        cwd=PACKAGE_ROOT,
        check=True,
        capture_output=True,
    )


def test_wheel_ships_no_test_files(tmp_path):
    _build(tmp_path)
    wheel = next(tmp_path.glob("*.whl"))
    names = zipfile.ZipFile(wheel).namelist()
    assert [n for n in names if n.endswith("_test.py")] == []


def test_sdist_ships_no_test_files(tmp_path):
    _build(tmp_path)
    sdist = next(tmp_path.glob("*.tar.gz"))
    names = tarfile.open(sdist, "r:gz").getnames()
    assert [n for n in names if n.endswith("_test.py")] == []
