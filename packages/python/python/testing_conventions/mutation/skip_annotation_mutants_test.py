"""Unit tests for the skip that keeps an annotation mutant from being judged."""
from types import SimpleNamespace

from testing_conventions.mutation.skip_annotation_mutants import (
    read_module_source,
    skip_annotation_mutants,
)

SOURCE = "def add(a: int | None, b: int = 10 - 4) -> int:\n    return a or b\n"


def _mutation(start_pos, end_pos, module_path="calc.py"):
    return SimpleNamespace(module_path=module_path, start_pos=start_pos, end_pos=end_pos)


def _item(job_id, *mutations):
    return SimpleNamespace(job_id=job_id, mutations=list(mutations))


def test_the_mutation_inside_an_annotation_is_skipped_and_the_default_is_left_alone(cosmic_ray):
    # Column 15 is the `|` of `int | None`; column 35 is the `-` of the `10 - 4` default, live
    # code on the same line, so a line-scoped drop would take it too.
    cosmic_ray.db.work_items = [
        _item("annotation", _mutation((1, 15), (1, 16))),
        _item("default", _mutation((1, 35), (1, 36))),
    ]

    skipped = skip_annotation_mutants(cosmic_ray.db, read_source=lambda path: SOURCE)

    assert skipped == ["annotation"]
    cosmic_ray.WorkResult.assert_called_once_with(worker_outcome="skipped")
    cosmic_ray.db.set_result.assert_called_once_with(
        "annotation", cosmic_ray.WorkResult.return_value
    )


def test_each_module_is_read_once_however_many_mutations_it_carries(cosmic_ray):
    reads = []

    def read_source(path):
        reads.append(path)
        return SOURCE

    cosmic_ray.db.work_items = [
        _item("parameter", _mutation((1, 15), (1, 16))),
        _item("returns", _mutation((1, 43), (1, 46))),
    ]

    skipped = skip_annotation_mutants(cosmic_ray.db, read_source=read_source)

    assert skipped == ["parameter", "returns"]
    assert reads == ["calc.py"]


def test_an_item_is_skipped_when_any_of_its_mutations_sits_in_an_annotation(cosmic_ray):
    cosmic_ray.db.work_items = [
        _item("mixed", _mutation((2, 13), (2, 15)), _mutation((1, 15), (1, 16)))
    ]

    assert skip_annotation_mutants(cosmic_ray.db, read_source=lambda path: SOURCE) == ["mixed"]


def test_a_module_without_annotations_keeps_every_mutant(cosmic_ray):
    cosmic_ray.db.work_items = [_item("body", _mutation((2, 13), (2, 15)))]

    skipped = skip_annotation_mutants(
        cosmic_ray.db, read_source=lambda path: "def add(a, b):\n    return a or b\n"
    )

    assert skipped == []
    cosmic_ray.db.set_result.assert_not_called()


def test_the_default_reader_decodes_the_module_as_utf8(tmp_path):
    path = tmp_path / "calc.py"
    path.write_text("def f(ñ: int) -> None:\n", encoding="utf-8")

    assert read_module_source(str(path)) == "def f(ñ: int) -> None:\n"
