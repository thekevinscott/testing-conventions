"""Unit tests for judging each mutant by its colocated test first."""
from enum import Enum
from types import SimpleNamespace

import pytest

from testing_conventions.mutation.execute_colocated_first import execute_colocated_first


class Outcome(Enum):
    KILLED = "killed"
    SURVIVED = "survived"


def _config():
    return SimpleNamespace(test_command="pytest -x", timeout=7.0)


def _item(job_id, module_path):
    return SimpleNamespace(job_id=job_id, mutations=[SimpleNamespace(module_path=module_path)])


def _result(outcome, output=""):
    return SimpleNamespace(test_outcome=outcome, output=output)


@pytest.fixture
def module(tmp_path):
    (tmp_path / "calc.py").write_text("")
    (tmp_path / "calc_test.py").write_text("")
    return tmp_path


def test_a_kill_by_the_colocated_test_is_the_result(cosmic_ray, module):
    item = _item("job", module / "calc.py")
    cosmic_ray.db.pending_work_items = (item,)
    killed = _result(Outcome.KILLED, "1 failed")
    cosmic_ray.mutate_and_test.return_value = killed

    execute_colocated_first(cosmic_ray.db, _config())

    cosmic_ray.mutate_and_test.assert_called_once_with(
        item.mutations, f"pytest -x {module / 'calc_test.py'}", 7.0
    )
    cosmic_ray.db.set_result.assert_called_once_with("job", killed)


def test_a_survivor_of_the_colocated_test_reruns_under_the_full_suite(cosmic_ray, module):
    item = _item("job", module / "calc.py")
    cosmic_ray.db.pending_work_items = (item,)
    full = _result("survived")
    cosmic_ray.mutate_and_test.side_effect = [_result("survived"), full]

    execute_colocated_first(cosmic_ray.db, _config())

    assert cosmic_ray.mutate_and_test.call_args_list[1].args == (item.mutations, "pytest -x", 7.0)
    cosmic_ray.db.set_result.assert_called_once_with("job", full)


def test_a_colocated_timeout_reruns_under_the_full_suite(cosmic_ray, module):
    cosmic_ray.db.pending_work_items = (_item("job", module / "calc.py"),)
    full = _result("killed", "1 failed")
    cosmic_ray.mutate_and_test.side_effect = [_result("killed", "timeout"), full]

    execute_colocated_first(cosmic_ray.db, _config())

    cosmic_ray.db.set_result.assert_called_once_with("job", full)


def test_a_module_with_no_colocated_test_runs_the_full_suite_alone(cosmic_ray, tmp_path):
    item = _item("job", tmp_path / "calc.py")
    cosmic_ray.db.pending_work_items = (item,)

    execute_colocated_first(cosmic_ray.db, _config())

    cosmic_ray.run_tests.assert_not_called()
    cosmic_ray.mutate_and_test.assert_called_once_with(item.mutations, "pytest -x", 7.0)


def test_a_colocated_test_failing_unmutated_never_judges(cosmic_ray, module):
    item = _item("job", module / "calc.py")
    cosmic_ray.db.pending_work_items = (item,)
    cosmic_ray.run_tests.return_value = (Outcome.KILLED, "1 failed")

    execute_colocated_first(cosmic_ray.db, _config())

    cosmic_ray.mutate_and_test.assert_called_once_with(item.mutations, "pytest -x", 7.0)


def test_the_clean_check_runs_once_per_colocated_test(cosmic_ray, module):
    cosmic_ray.db.pending_work_items = (
        _item("a", module / "calc.py"),
        _item("b", module / "calc.py"),
    )
    cosmic_ray.run_tests.return_value = (Outcome.SURVIVED, "")
    cosmic_ray.mutate_and_test.return_value = _result(Outcome.KILLED, "1 failed")

    execute_colocated_first(cosmic_ray.db, _config())

    cosmic_ray.run_tests.assert_called_once_with(f"pytest -x {module / 'calc_test.py'}", 7.0)
    assert cosmic_ray.mutate_and_test.call_count == 2


def test_a_colocated_path_is_shell_quoted(cosmic_ray, tmp_path):
    spaced = tmp_path / "my pkg"
    spaced.mkdir()
    (spaced / "calc.py").write_text("")
    (spaced / "calc_test.py").write_text("")
    cosmic_ray.db.pending_work_items = (_item("job", spaced / "calc.py"),)
    cosmic_ray.mutate_and_test.return_value = _result("killed", "1 failed")

    execute_colocated_first(cosmic_ray.db, _config())

    cosmic_ray.run_tests.assert_called_once_with(f"pytest -x '{spaced / 'calc_test.py'}'", 7.0)
