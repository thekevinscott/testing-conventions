import pytest


@pytest.fixture
def unused_client():
    # Only runs when a test requests the fixture; nothing does, so this line is
    # uncovered. If conftest.py were counted as a subject instead of omitted, it
    # would drag the codebase below a 100 floor — widget.py alone is fully covered.
    return object()
