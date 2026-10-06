"""Unit tests for the mutation adapter's cosmic-ray session."""
from pathlib import Path
from types import SimpleNamespace


class _Config(dict):
    operators_config = {"core/NumberReplacer": {}}


def _config(module_path):
    cfg = _Config()
    cfg["module-path"] = module_path
    cfg["excluded-modules"] = ["*_test.py"]
    return cfg


def test_inits_executes_and_returns_first_mutation_with_result(cosmic_ray):
    from testing_conventions.mutation.session import run_session

    mutation = SimpleNamespace(module_path="calc.py", start_pos=(2, 0), operator_name="Op")
    other = SimpleNamespace(module_path="calc.py", start_pos=(9, 0), operator_name="Other")
    result = SimpleNamespace(test_outcome="survived")
    cosmic_ray.find_modules.return_value = [Path("calc.py")]
    # Two mutations on the item so the *first* index is asserted, not just "a" mutation.
    cosmic_ray.db.completed_work_items = [(SimpleNamespace(mutations=[mutation, other]), result)]

    pairs = run_session(_config(["."]))

    assert pairs == [(mutation, result)]
    cosmic_ray.find_modules.assert_called_once_with([Path(".")])
    cosmic_ray.filter_paths.assert_called_once_with([Path("calc.py")], ["*_test.py"])
    cosmic_ray.init.assert_called_once()
    cosmic_ray.execute.assert_called_once()


def test_a_string_module_path_is_wrapped_in_a_list(cosmic_ray):
    from testing_conventions.mutation.session import run_session

    cosmic_ray.find_modules.return_value = []
    cosmic_ray.db.completed_work_items = []
    run_session(_config("calc.py"))
    cosmic_ray.find_modules.assert_called_once_with([Path("calc.py")])


def test_a_mutation_inside_an_annotation_is_skipped_before_execute(cosmic_ray, tmp_path):
    from testing_conventions.mutation.session import run_session

    module = tmp_path / "calc.py"
    module.write_text("def add(a: int | None, b: int) -> int:\n    return a or b\n")
    # Line 1 column 15 is the `|` of `int | None`, so the engine never judges this work item:
    # the skip records its result, and `execute` schedules only the items carrying none.
    mutation = SimpleNamespace(module_path=str(module), start_pos=(1, 15), end_pos=(1, 16))
    cosmic_ray.db.work_items = [SimpleNamespace(job_id="annotation", mutations=[mutation])]
    cosmic_ray.db.completed_work_items = []

    assert run_session(_config([str(tmp_path)])) == []

    cosmic_ray.db.set_result.assert_called_once_with(
        "annotation", cosmic_ray.WorkResult.return_value
    )
    cosmic_ray.execute.assert_called_once()
