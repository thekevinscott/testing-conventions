# A legacy `test_*.py` name. Under the greenfield convention (#112) this is
# ordinary source, not a test file, so the integration lints never scan it — even
# though it declares pytest's `monkeypatch`, which would be a `no-monkeypatch`
# violation in a real test file (`*_test.py`).
import os


def describe_widget():
    def it_reads_the_token(monkeypatch):
        monkeypatch.setenv("MYPROJECT_TOKEN", "test-token")
        assert os.environ["MYPROJECT_TOKEN"] == "test-token"
