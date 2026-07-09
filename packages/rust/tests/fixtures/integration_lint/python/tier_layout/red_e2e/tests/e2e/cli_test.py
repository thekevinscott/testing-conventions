# A test that takes pytest's `monkeypatch` fixture — the integration lints
# must flag it even though the call's `path` is the sibling `src/` directory.
import os


def describe_flow():
    def it_reads_the_token(monkeypatch):
        monkeypatch.setenv("MYPROJECT_TOKEN", "test-token")
        assert os.environ["MYPROJECT_TOKEN"] == "test-token"
