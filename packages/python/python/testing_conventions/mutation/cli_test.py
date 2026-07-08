"""Unit tests for the mutation adapter's orchestrator.

These drive the real ``mutation_cli`` over the fake ``cosmic_ray`` engine — parse args, build
config, baseline, session, normalize, write — so the whole flow is exercised end-to-end
without the engine installed.
"""
import json
from types import SimpleNamespace

import pytest

from testing_conventions.mutation.cli import mutation_cli


class _Config(dict):
    operators_config = {}


def _config():
    cfg = _Config()
    cfg["module-path"] = ["."]
    cfg["excluded-modules"] = []
    return cfg


def _survivor():
    mutation = SimpleNamespace(module_path="calc.py", start_pos=(6, 0), operator_name="Op")
    return (SimpleNamespace(mutations=[mutation]), SimpleNamespace(test_outcome="survived"))


def _baseline(outcome, output=""):
    return iter([("baseline", SimpleNamespace(test_outcome=outcome, output=output))])


def test_writes_the_normalized_results_to_out(cosmic_ray, tmp_path):
    cosmic_ray.deserialize_config.return_value = _config()
    cosmic_ray.db.results = _baseline("survived")
    cosmic_ray.find_modules.return_value = []
    cosmic_ray.db.completed_work_items = [_survivor()]
    out = tmp_path / "r.json"

    mutation_cli(["--out", str(out)])

    assert json.loads(out.read_text()) == [
        {"file": "calc.py", "line": 6, "status": "survived", "mutator": "Op"}
    ]
    # The clean suite is measured under the generous ceiling (300s), then each mutant's
    # timeout is scoped to the observed runtime — a near-instant fake run floors to 10s — rather
    # than a fixed 30s. Both rendered configs reach cosmic-ray, proving the two-timeout wiring.
    rendered = [call.args[0] for call in cosmic_ray.deserialize_config.call_args_list]
    assert any("timeout = 300.0\n" in toml for toml in rendered)
    assert any("timeout = 10.0\n" in toml for toml in rendered)


def test_a_baseline_failure_exits_nonzero_and_writes_nothing(cosmic_ray, tmp_path, capsys):
    cosmic_ray.deserialize_config.return_value = _config()
    cosmic_ray.db.results = _baseline("killed", output="boom")
    out = tmp_path / "r.json"

    with pytest.raises(SystemExit) as exc:
        mutation_cli(["--out", str(out)])

    assert exc.value.code == 1
    assert "did not pass unmutated" in capsys.readouterr().err
    assert not out.exists()
