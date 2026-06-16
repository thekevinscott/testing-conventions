# Waived fixture for R1 (#49): identical to the red fixture (takes pytest's
# `monkeypatch`), but waived in the colocated testing-conventions.toml — so it
# must be silent.
import os


def describe_widget():
    def it_reads_the_token(monkeypatch):
        monkeypatch.setenv("MYPROJECT_TOKEN", "test-token")
        assert os.environ["MYPROJECT_TOKEN"] == "test-token"
