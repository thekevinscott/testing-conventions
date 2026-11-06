"""Judge each mutant by its colocated test first, and by the full suite when that is not a kill.

``cosmic_ray`` is imported lazily (the module imports without the engine; it is unit-tested with a
fake ``cosmic_ray`` injected).
"""
from __future__ import annotations

import shlex

from testing_conventions.mutation.colocated_test_path import colocated_test_path

# cosmic-ray reports a timed-out run as a kill whose output is this word.
TIMED_OUT = frozenset({"timeout"})


def execute_colocated_first(database, config):
    """Record a result for every pending work item in ``database``: the colocated test's kill, or
    else the full ``config.test_command`` run. A colocated test that fails unmutated never judges."""
    from cosmic_ray.mutating import mutate_and_test
    from cosmic_ray.testing import run_tests

    full = config.test_command
    timeout = config.timeout
    passes_clean = {}

    def outcome(result):
        return getattr(result.test_outcome, "value", result.test_outcome)

    def narrowed_command(item):
        mutation, *_ = item.mutations
        test = colocated_test_path(mutation.module_path)
        if test is None:
            return None
        command = f"{full} {shlex.quote(str(test))}"
        if test not in passes_clean:
            status, _ = run_tests(command, timeout)
            passes_clean[test] = getattr(status, "value", status) in {"survived"}
        return command if passes_clean[test] else None

    for item in database.pending_work_items:
        command = narrowed_command(item)
        result = command and mutate_and_test(item.mutations, command, timeout)
        if not (result and outcome(result) in {"killed"} and result.output not in TIMED_OUT):
            result = mutate_and_test(item.mutations, full, timeout)
        database.set_result(item.job_id, result)
