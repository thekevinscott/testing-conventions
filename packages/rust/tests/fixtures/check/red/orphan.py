def helper():
    # No colocated orphan_test.py, so `unit colocated-test (python)` fails — and
    # the umbrella fails with it, even though the TypeScript pair below is clean.
    return 42
