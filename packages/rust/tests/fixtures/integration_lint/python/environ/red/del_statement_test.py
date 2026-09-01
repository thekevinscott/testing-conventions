# Red fixture for `no-environ-mutation`: a test deletes from os.environ.
import os


def describe_widget():
    def it_clears_a_token():
        del os.environ["MYPROJECT_TOKEN"]
        assert run() is None
