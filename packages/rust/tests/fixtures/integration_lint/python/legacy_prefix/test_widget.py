# Red fixture for #145: a *legacy* `test_*.py` carrying a `no-monkeypatch` violation
# (it takes pytest's `monkeypatch` fixture). After #112 the greenfield layout makes a
# unit test `*_test.py` and treats a `test_*.py` as ordinary *source*, so the
# integration lints must NOT scan this file — even though the same body in a
# `*_test.py` would be flagged (see `monkeypatch/red`).
import os


def describe_widget():
    def it_reads_the_token(monkeypatch):
        monkeypatch.setenv("MYPROJECT_TOKEN", "test-token")
        assert os.environ["MYPROJECT_TOKEN"] == "test-token"
