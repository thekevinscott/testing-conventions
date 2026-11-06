"""Keep the engine from running a mutant that sits off the diff's changed lines."""
from __future__ import annotations


def skip_off_line_mutants(database, ranges):
    """Record a skipped result for every work item in ``database`` whose mutation sits off
    ``ranges`` (file to inclusive line ranges), and return the job ids skipped. Empty ``ranges``
    scopes nothing. The file and line are the ones ``normalize`` reports, so the rust core's
    changed-line filter would have dropped each skipped item anyway."""
    from cosmic_ray.work_item import WorkerOutcome, WorkResult

    if not ranges:
        return []
    skipped = []
    for item in database.work_items:
        mutation = item.mutations[0]
        line = mutation.start_pos[0]
        spans = ranges.get(str(mutation.module_path).replace("\\", "/"), ())
        if not any(start <= line <= end for start, end in spans):
            database.set_result(item.job_id, WorkResult(worker_outcome=WorkerOutcome.SKIPPED))
            skipped.append(item.job_id)
    return skipped
