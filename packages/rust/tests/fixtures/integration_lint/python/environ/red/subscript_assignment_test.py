# Red fixture for R3 (#51): a test mutates os.environ by subscript assignment.
# Set env via patch.dict(os.environ, {...}) in a fixture instead.
import os


def describe_widget():
    def it_sets_a_token():
        os.environ["MYPROJECT_TOKEN"] = "test-token"
        assert run() == "test-token"
