"""Generate the packaging-package-root self-test tarball fixtures (#280).

Two minimal `npm pack`-shaped tarballs (a gzipped tar with a top-level `package/` dir), proving
the packaging gate discovers a built distribution at the *derived package root* rather than only
the checkout root:

  clean/dist/packaging-package-root-fixture-0.0.0.tgz — no test file; ships alongside the
      fixture's own `package.json`, so a per-package `uses:` call scoped to `clean/` finds and
      passes it with no input beyond `source` (`packaging-package-root-clean` in
      testing-conventions-selftest.yml).
  red/dist/packaging-package-root-fixture-0.0.0.tgz — ships `package/dist/widget.test.js`, so
      the published `packaging` command fails on it directly (a `uses:` call that fails would
      fail the whole self-test run — see `packaging-red` for the same convention). Kept at a
      `dist/` path too, mirroring the clean fixture and the shape a real per-package build
      produces.

Regenerate:  python make_fixtures.py
"""

import gzip
import io
import tarfile
from pathlib import Path

HERE = Path(__file__).parent
NAME = "packaging-package-root-fixture-0.0.0.tgz"

PKG_JSON = (
    '{\n  "name": "packaging-package-root-fixture",\n  "version": "0.0.0",\n'
    '  "main": "dist/widget.js"\n}\n'
)
SOURCE = "export const widget = () => 1;\n"
TEST = "import { widget } from './widget';\ntest('widget', () => expect(widget()).toBe(1));\n"

COMMON = {
    "package/package.json": PKG_JSON,
    "package/dist/widget.js": SOURCE,
}


def write_tarball(path: Path, files: dict) -> None:
    # Fixed mtimes (tar entries + gzip header) so regenerating is byte-stable.
    path.parent.mkdir(parents=True, exist_ok=True)
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


write_tarball(HERE / "clean" / "dist" / NAME, COMMON)
write_tarball(HERE / "red" / "dist" / NAME, {**COMMON, "package/dist/widget.test.js": TEST})
print("wrote clean/dist and red/dist tarballs")
