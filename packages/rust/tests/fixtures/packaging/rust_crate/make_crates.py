"""Generate the packaging Rust-crate fixtures for #74.

`cargo package` produces `name-version.crate` — a gzipped tar wrapping a single
`name-version/` dir (Cargo.toml, src/, and the crate-root `tests/` unless a Cargo
`exclude` keeps it out). Two minimal ones:

  widget-0.1.0.crate — ships the crate-root `tests/integration.rs` (an integration
                       test that should have been excluded); the rule must flag it.
  clean-0.1.0.crate  — the same crate with `exclude = ["tests/**"]`, so the
                       tarball has no `tests/`; the rule must pass.

Inline `#[cfg(test)]` units compile out of the consumer artifact for free, so the
only thing to check in the source tarball is the crate-root `tests/` directory.
The checker unpacks the `.crate` (a gzipped tar) and flags files under `tests/`.

Regenerate:  python make_crates.py
"""

import gzip
import io
import tarfile
from pathlib import Path

HERE = Path(__file__).parent

CARGO_TOML = '[package]\nname = "widget"\nversion = "0.1.0"\nedition = "2021"\n'
SOURCE = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n"
INTEGRATION_TEST = "#[test]\nfn adds() {\n    assert_eq!(widget::add(1, 2), 3);\n}\n"


def write_crate(path: Path, root: str, files: dict) -> None:
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


common = {"Cargo.toml": CARGO_TOML, "src/lib.rs": SOURCE}
# red: the crate-root tests/ leaked into the package; clean: excluded.
write_crate(HERE / "widget-0.1.0.crate", "widget-0.1.0", {**common, "tests/integration.rs": INTEGRATION_TEST})
write_crate(HERE / "clean-0.1.0.crate", "clean-0.1.0", common)
print("wrote widget-0.1.0.crate and clean-0.1.0.crate")
