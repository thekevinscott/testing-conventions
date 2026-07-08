"""Unit test for the mutation adapter's executable entry.

Importing ``main`` runs ``mutation_cli(sys.argv[1:])`` at module top (the same top-level run
``python -m testing_conventions.mutation.main`` performs), so the test sets ``sys.argv`` and a
fake engine, imports the module fresh, and asserts it wrote the normalized results.
"""
import importlib
import json
import sys
from types import SimpleNamespace


class _Config(dict):
    operators_config = {}


def test_running_the_module_invokes_the_cli_over_argv(cosmic_ray, tmp_path):
    out = tmp_path / "r.json"
    config = _Config()
    config["module-path"] = ["."]
    config["excluded-modules"] = []
    cosmic_ray.deserialize_config.return_value = config
    cosmic_ray.db.results = iter([("baseline", SimpleNamespace(test_outcome="survived", output=""))])
    cosmic_ray.find_modules.return_value = []
    mutation = SimpleNamespace(module_path="calc.py", start_pos=(2, 0), operator_name="Op")
    cosmic_ray.db.completed_work_items = [
        (SimpleNamespace(mutations=[mutation]), SimpleNamespace(test_outcome="survived"))
    ]

    saved_argv = sys.argv
    sys.argv = ["python", "--out", str(out)]
    try:
        sys.modules.pop("testing_conventions.mutation.main", None)
        importlib.import_module("testing_conventions.mutation.main")
    finally:
        sys.argv = saved_argv
        sys.modules.pop("testing_conventions.mutation.main", None)

    assert json.loads(out.read_text()) == [
        {"file": "calc.py", "line": 2, "status": "survived", "mutator": "Op"}
    ]
