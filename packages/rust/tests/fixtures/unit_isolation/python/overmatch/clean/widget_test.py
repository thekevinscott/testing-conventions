# Clean fixture for #393: a multi-symbol collaborator import is mocked when EVERY
# imported symbol is patched at that import's own module path
# (`myproject.ledger.record` and `myproject.ledger.erase`). No symbol is left real, so
# there is no violation.
import pytest

from myproject.widget import build  # unit under test

from myproject.ledger import record, erase  # both mocked below at their module path


@pytest.fixture(autouse=True)
def mock_ledger(mocker):
    mocker.patch("myproject.ledger.record")
    mocker.patch("myproject.ledger.erase")


def describe_build():
    def it_builds():
        assert build(record, erase) is not None
