"""Keep the engine from judging a mutation that sits inside a type annotation."""
from __future__ import annotations

from pathlib import Path

from testing_conventions.mutation.annotation_spans import annotation_spans


def read_module_source(path):
    """The text of the module at ``path``, decoded as utf-8 — Python's own source encoding."""
    return Path(path).read_text(encoding="utf-8")


def skip_annotation_mutants(database, read_source=read_module_source):
    """Record a skipped result for every work item in ``database`` whose mutation sits inside an
    annotation, reading each module through ``read_source``, and return the job ids skipped.
    ``execute`` schedules only the items that carry no result, so such a mutant never runs a
    suite, and a skipped result holds no test outcome, so ``normalize`` drops it unjudged."""
    from cosmic_ray.work_item import WorkerOutcome, WorkResult

    spans = {}

    def inside_an_annotation(mutation):
        path = str(mutation.module_path)
        if path not in spans:
            spans[path] = annotation_spans(read_source(path))
        return any(
            start <= mutation.start_pos and mutation.end_pos <= end
            for start, end in spans[path]
        )

    skipped = []
    for item in database.work_items:
        if any(inside_an_annotation(mutation) for mutation in item.mutations):
            database.set_result(item.job_id, WorkResult(worker_outcome=WorkerOutcome.SKIPPED))
            skipped.append(item.job_id)
    return skipped
