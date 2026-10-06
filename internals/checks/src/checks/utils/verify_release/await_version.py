"""Wait for npm to serve a just-published version."""
from __future__ import annotations

import subprocess
import time

VERSION_VISIBLE_TIMEOUT_S = 900
VERSION_POLL_INTERVAL_S = 15


def await_version(version: str, run=subprocess.run, sleep=time.sleep, clock=time.monotonic) -> None:
    """Poll npm until `testing-conventions@version` installs and runs, the way a check job runs it.

    The registry lists a version in its metadata minutes before the tarball and the platform
    binaries reach every CDN edge, so only a real install proves the version is fetchable."""
    argv = ["npm", "exec", "--yes", "--prefer-online", "--", f"testing-conventions@{version}", "--help"]
    deadline = clock() + VERSION_VISIBLE_TIMEOUT_S
    while run(argv, capture_output=True).returncode:
        if clock() >= deadline:
            raise TimeoutError(
                f"testing-conventions@{version} was not installable from npm within "
                f"{VERSION_VISIBLE_TIMEOUT_S}s"
            )
        sleep(VERSION_POLL_INTERVAL_S)
