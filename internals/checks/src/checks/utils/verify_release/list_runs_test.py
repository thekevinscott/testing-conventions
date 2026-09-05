"""Colocated unit tests for the run listing (isolation — an injected `run` fake)."""
import json

from checks.utils.verify_release.list_runs import list_runs


class _Result:
    def __init__(self, stdout="", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


def test_list_runs_asks_gh_for_the_workflow_runs_and_parses_the_json():
    calls = []
    rows = [{"databaseId": 42, "headSha": "sha", "event": "workflow_dispatch"}]

    def run(argv, **kwargs):
        calls.append(argv)
        return _Result(stdout=json.dumps(rows))

    assert list_runs("a.yml", run) == rows
    assert calls == [[
        "gh", "run", "list", "--workflow", "a.yml", "--limit", "40",
        "--json", "databaseId,headSha,event,status,conclusion,createdAt",
    ]]
