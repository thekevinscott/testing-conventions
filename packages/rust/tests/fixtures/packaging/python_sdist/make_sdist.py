"""Generate the packaging Python-sdist fixtures for #106.

A Python sdist is a gzipped tar (`name-version.tar.gz`) with a top-level
`name-version/` dir. Two minimal ones stand in for a consumer's source
distribution:

  widget-0.1.0.tar.gz — ships a colocated test (`widget/core_test.py`) that
                        should have been excluded from the sdist; the rule must
                        flag it.
  clean-0.1.0.tar.gz  — the same shape without it; the rule must pass.

The packaging checker unpacks the `.tar.gz` (#73 added tar.gz support) and scans
for `*_test.py`. These don't need to be installable — only to contain (or not) a
test file.

Regenerate:  python make_sdist.py
"""

import gzip
import io
import tarfile
from pathlib import Path

HERE = Path(__file__).parent

PKG_INFO = "Metadata-Version: 2.1\nName: widget\nVersion: 0.1.0\n"
SOURCE = "def add(a, b):\n    return a + b\n"
TEST = "from widget.core import add\n\n\ndef test_add():\n    assert add(1, 2) == 3\n"


def write_sdist(path: Path, root: str, files: dict) -> None:
    # Fixed mtimes (tar entries + gzip header) so regenerating is byte-stable.
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w") as tar:
        for name, content in sorted(files.items()):
            data = content.encode()
            info = tarfile.TarInfo(f"{root}/{name}")
            info.size = len(data)
            info.mtime = 0
            tar.addfile(info, io.BytesIO(data))
    with gzip.GzipFile(path, "wb", mtime=0) as gz:
        gz.write(raw.getvalue())


common = {
    "PKG-INFO": PKG_INFO,
    "widget/__init__.py": "",
    "widget/core.py": SOURCE,
}
write_sdist(HERE / "widget-0.1.0.tar.gz", "widget-0.1.0", {**common, "widget/core_test.py": TEST})
write_sdist(HERE / "clean-0.1.0.tar.gz", "clean-0.1.0", common)
print("wrote widget-0.1.0.tar.gz and clean-0.1.0.tar.gz")
