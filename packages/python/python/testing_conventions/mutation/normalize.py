"""Map a cosmic-ray work item + result onto the normalized mutation schema.

The rust core gates over one engine-agnostic representation
(``{file, line, status, mutator}``); this turns cosmic-ray's per-mutant outcome into it,
the Python counterpart of the TypeScript adapter's ``to-normalized``. It reads only plain
attributes off the passed objects, so it needs no ``cosmic_ray`` import.
"""
from __future__ import annotations

# cosmic-ray's ``TestOutcome`` values → the normalized ``MutantStatus`` vocabulary.
# ``survived`` / ``killed`` map straight across; ``incompetent`` (the mutation produced code
# the interpreter rejected — a syntax / import error, never a viable mutant) is
# ``compile_error``. cosmic-ray has no distinct no-coverage outcome: an uncovered mutant's
# suite still passes, so it reports ``survived``.
STATUS = {"survived": "survived", "killed": "killed", "incompetent": "compile_error"}


def normalize(mutation, result):
    """Return the normalized mutant dict for one completed work item, or ``None`` to skip a
    work item with no usable outcome (the worker never judged it — abnormal / no test)."""
    outcome = result.test_outcome
    status = STATUS.get(getattr(outcome, "value", outcome))
    if status is None:
        return None
    return {
        "file": str(mutation.module_path).replace("\\", "/"),
        "line": mutation.start_pos[0],
        "status": status,
        "mutator": mutation.operator_name,
    }
