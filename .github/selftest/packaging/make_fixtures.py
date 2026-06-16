"""Generate the packaging self-test wheel fixtures.

Two minimal wheels (a wheel is a zip) for the reusable workflow's `packaging` job:

  clean/widget-0.1.0-py3-none-any.whl — no test file; the check passes.
  red/widget-0.1.0-py3-none-any.whl   — ships widget/core_test.py; the check fails.

`testing-conventions-selftest.yml` uploads `clean/` as an artifact and runs the
reusable workflow's packaging job over it (must pass), and drives the published
CLI on the red wheel directly (must fail).

Regenerate:  python make_fixtures.py
"""

import zipfile
from pathlib import Path

HERE = Path(__file__).parent
NAME = "widget-0.1.0-py3-none-any.whl"

METADATA = "Metadata-Version: 2.1\nName: widget\nVersion: 0.1.0\n"
WHEEL = (
    "Wheel-Version: 1.0\nGenerator: make_fixtures.py\n"
    "Root-Is-Purelib: true\nTag: py3-none-any\n"
)
SOURCE = "def add(a, b):\n    return a + b\n"
TEST = "from widget.core import add\n\n\ndef test_add():\n    assert add(1, 2) == 3\n"

COMMON = {
    "widget/__init__.py": "",
    "widget/core.py": SOURCE,
    "widget-0.1.0.dist-info/METADATA": METADATA,
    "widget-0.1.0.dist-info/WHEEL": WHEEL,
}


def write_wheel(path: Path, files: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as zf:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            zf.writestr(info, content)


write_wheel(HERE / "clean" / NAME, COMMON)
write_wheel(HERE / "red" / NAME, {**COMMON, "widget/core_test.py": TEST})
print("wrote clean/ and red/ wheels")
