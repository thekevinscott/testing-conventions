# Red fixture for R3 (#51): a test mutates os.environ via a method call.
import os


def describe_widget():
    def it_updates_env():
        os.environ.update({"MYPROJECT_TOKEN": "test-token"})
        assert run() == "test-token"
