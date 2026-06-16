# Red fixture for Python unit isolation, external deps (#121, slice 3). A unit test
# must mock external collaborators too — a third-party package and effectful stdlib
# imported and used for real are violations; pure stdlib is fine.
from myproject.widget import build  # unit under test — never a collaborator

import requests  # third-party — NOT mocked → violation
import subprocess  # effectful stdlib (subprocess) — NOT mocked → violation
import json  # pure stdlib — fine, never a collaborator


def describe_build():
    def it_builds():
        # Uses the real collaborators — not isolated.
        assert build(requests, subprocess, json) is not None
