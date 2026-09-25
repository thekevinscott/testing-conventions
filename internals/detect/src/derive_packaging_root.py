from pathlib import Path

# `cargo package` writes the crate here; every other layout puts the distribution in `dist/`. A dict
# lookup rather than `primary == "rust"`, which mutates to an unkillable `is` on an interned literal.
_SUBDIR = {"rust": "target/package"}


def derive_packaging_root(package_root_rel: Path, primary: str) -> str:
    """The repo-root-relative directory the packaging scan is pointed at.

    Derived from the primary language rather than from `packaging_build`, because the packaging job
    also runs on a committed `dist/` whose manifest states no build — an empty root there would
    fail a path the `packaging_dist` gate exists to let pass.
    """
    return f"{package_root_rel}/{_SUBDIR.get(primary, 'dist')}"
