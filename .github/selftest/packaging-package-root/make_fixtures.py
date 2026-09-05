"""Generate the packaging-package-root self-test tarball fixtures: two `npm pack`-shaped
tarballs (gzipped tar, top-level `package/` dir); clean/ ships no test file, red/ ships
`package/dist/widget.test.js`. Regenerate:  python make_fixtures.py
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
