"""Unit tests for the skip that keeps an off-line mutant from running the suite."""
from pathlib import PureWindowsPath
from types import SimpleNamespace

from testing_conventions.mutation.skip_off_line_mutants import skip_off_line_mutants


def _item(job_id, line, module_path="calc.py"):
    mutation = SimpleNamespace(module_path=module_path, start_pos=(line, 4))
    return SimpleNamespace(job_id=job_id, mutations=[mutation])


def test_items_off_the_ranges_are_skipped_and_the_bounds_are_kept(cosmic_ray):
    cosmic_ray.db.work_items = [
        _item("before", 2),
        _item("start", 3),
        _item("end", 6),
        _item("after", 7),
    ]

    skipped = skip_off_line_mutants(cosmic_ray.db, {"calc.py": [(3, 6)]})

    assert skipped == ["before", "after"]
    cosmic_ray.WorkResult.assert_called_with(worker_outcome="skipped")
    assert [call.args[0] for call in cosmic_ray.db.set_result.call_args_list] == [
        "before",
        "after",
    ]


def test_a_module_with_no_ranges_has_every_item_skipped(cosmic_ray):
    cosmic_ray.db.work_items = [_item("other", 3, module_path="other.py")]

    assert skip_off_line_mutants(cosmic_ray.db, {"calc.py": [(3, 6)]}) == ["other"]


def test_any_range_of_a_module_keeps_the_item(cosmic_ray):
    cosmic_ray.db.work_items = [_item("second", 9)]

    assert skip_off_line_mutants(cosmic_ray.db, {"calc.py": [(3, 6), (9, 9)]}) == []


def test_the_first_mutation_decides_as_it_is_the_one_reported(cosmic_ray):
    reported, trailing = _item("first", 3).mutations[0], _item("last", 9).mutations[0]
    cosmic_ray.db.work_items = [SimpleNamespace(job_id="pair", mutations=[reported, trailing])]

    assert skip_off_line_mutants(cosmic_ray.db, {"calc.py": [(3, 3)]}) == []


def test_the_module_path_is_matched_with_forward_slashes(cosmic_ray):
    cosmic_ray.db.work_items = [_item("nested", 3, module_path=PureWindowsPath("pkg\\calc.py"))]

    assert skip_off_line_mutants(cosmic_ray.db, {"pkg/calc.py": [(3, 3)]}) == []


def test_no_ranges_scope_nothing(cosmic_ray):
    cosmic_ray.db.work_items = [_item("any", 3)]

    assert skip_off_line_mutants(cosmic_ray.db, {}) == []
    cosmic_ray.db.set_result.assert_not_called()
