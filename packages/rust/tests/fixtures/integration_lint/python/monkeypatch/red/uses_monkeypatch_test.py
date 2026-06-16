# Red fixture for R1 (#49): a test that takes pytest's `monkeypatch` fixture.
# The mocking checker must flag the `monkeypatch` parameter.
import os


def describe_widget():
    def it_reads_the_token(monkeypatch):
        monkeypatch.setenv("MYPROJECT_TOKEN", "test-token")
        assert os.environ["MYPROJECT_TOKEN"] == "test-token"
