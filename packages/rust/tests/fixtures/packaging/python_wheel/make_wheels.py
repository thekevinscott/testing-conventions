"""Generate the packaging wheel fixtures for #72.

Two minimal wheels (a wheel is just a zip) standing in for a consumer's built
distribution:

  red.whl   — ships a colocated test (`widget/core_test.py`) that should have
              been stripped at build time; the packaging rule must flag it.
  clean.whl — the same package with the test excluded; the rule must pass.

The packaging checker unzips the wheel and scans for `*_test.py`. These don't
need to be installable — only to contain (or not) a test file.

Regenerate:  python make_wheels.py
"""

import zipfile
from pathlib import Path

HERE = Path(__file__).parent

METADATA = "Metadata-Version: 2.1\nName: widget\nVersion: 0.1.0\n"
WHEEL = (
    "Wheel-Version: 1.0\nGenerator: make_wheels.py\n"
    "Root-Is-Purelib: true\nTag: py3-none-any\n"
)
SOURCE = "def add(a, b):\n    return a + b\n"
TEST = "from widget.core import add\n\n\ndef test_add():\n    assert add(1, 2) == 3\n"

# The package as it ships, minus the test file.
COMMON = {
    "widget/__init__.py": "",
    "widget/core.py": SOURCE,
    "widget-0.1.0.dist-info/METADATA": METADATA,
    "widget-0.1.0.dist-info/WHEEL": WHEEL,
}


def write_wheel(path: Path, files: dict) -> None:
    # Fixed timestamps so regenerating produces byte-stable archives.
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as zf:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            zf.writestr(info, content)


write_wheel(HERE / "red.whl", {**COMMON, "widget/core_test.py": TEST})
write_wheel(HERE / "clean.whl", COMMON)
print("wrote red.whl and clean.whl")
