# Same un-mocked external collaborators as the red fixture — but this file is waived
# in `testing-conventions.toml`, so `unmocked-collaborator` is lifted.
from myproject.widget import build

import requests
import subprocess


def describe_build():
    def it_builds():
        assert build(requests, subprocess) is not None
