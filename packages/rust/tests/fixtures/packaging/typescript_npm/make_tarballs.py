"""Generate the packaging npm-tarball fixtures for #73.

`npm pack` produces a gzipped tar (`.tgz`) with a top-level `package/` dir. Two
minimal ones stand in for a consumer's published package:

  red.tgz   — ships a test file (`package/dist/widget.test.js`) that should have
              been excluded from the published `dist`; the rule must flag it.
  clean.tgz — the same package without it; the rule must pass.

The packaging checker unpacks the tarball and scans for `*.test.*`. These don't
need to be installable — only to contain (or not) a test file.

Regenerate:  python make_tarballs.py
"""

import gzip
import io
import tarfile
from pathlib import Path

HERE = Path(__file__).parent

PKG_JSON = '{\n  "name": "widget",\n  "version": "0.1.0",\n  "main": "dist/widget.js"\n}\n'
SOURCE = "export const widget = () => 1;\n"
TEST = "import { widget } from './widget';\ntest('widget', () => expect(widget()).toBe(1));\n"

# The published package, minus the test file.
COMMON = {
    "package/package.json": PKG_JSON,
    "package/dist/widget.js": SOURCE,
}


def write_tarball(path: Path, files: dict) -> None:
    # Fixed mtimes (tar entries + gzip header) so regenerating is byte-stable.
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w") as tar:
        for name, content in sorted(files.items()):
            data = content.encode()
            info = tarfile.TarInfo(name)
            info.size = len(data)
            info.mtime = 0
            tar.addfile(info, io.BytesIO(data))
    with gzip.GzipFile(path, "wb", mtime=0) as gz:
        gz.write(raw.getvalue())


write_tarball(HERE / "red.tgz", {**COMMON, "package/dist/widget.test.js": TEST})
write_tarball(HERE / "clean.tgz", COMMON)
print("wrote red.tgz and clean.tgz")
