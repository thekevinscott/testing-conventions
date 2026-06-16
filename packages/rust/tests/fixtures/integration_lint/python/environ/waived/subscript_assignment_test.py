# Waived fixture for R3 (#51): identical to the red fixture (mutates os.environ
# by subscript assignment), but waived in the colocated testing-conventions.toml
# — so it must be silent.
import os


def describe_widget():
    def it_sets_a_token():
        os.environ["MYPROJECT_TOKEN"] = "test-token"
        assert run() == "test-token"
