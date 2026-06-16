"""First-party code an integration test must run for real.

`MAX_RETRIES` is a module-global config constant — the kind an integration test
must *not* `patch(...)` (inject config instead); `clamp` carries the one branch the
unit suite covers.
"""

MAX_RETRIES = 3


def clamp(n):
    if n > MAX_RETRIES:
        return MAX_RETRIES
    return n
