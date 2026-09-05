"""Map a cosmic-ray work item + result onto the normalized mutation schema.

The rust core gates over one engine-agnostic representation
(``{file, line, status, mutator, replacement?}``); this turns cosmic-ray's per-mutant outcome into it,
the Python counterpart of the TypeScript adapter's ``to-normalized``. It reads only plain
attributes off the passed objects, so it needs no ``cosmic_ray`` import.
"""
from __future__ import annotations

# cosmic-ray's ``TestOutcome`` values to the normalized ``MutantStatus`` vocabulary.
# ``incompetent`` means the interpreter rejected the mutation, never a viable mutant. cosmic-ray
# has no no-coverage outcome: an uncovered mutant's suite passes, so it reports ``survived``.
STATUS = {"survived": "survived", "killed": "killed", "incompetent": "compile_error"}


def replacement_from_diff(diff):
    """The source line(s) a work result's unified ``diff`` adds, or ``None`` when it adds none."""
    added = []
    in_hunk = False
    for line in (diff or "").splitlines():
        if line.startswith("@@"):
            in_hunk = True
        elif in_hunk and line.startswith("+"):
            added.append(line[1:].strip())
    return "\n".join(added) or None


def normalize(mutation, result):
    """Return the normalized mutant dict for one completed work item, or ``None`` to skip a
    work item with no usable outcome (the worker never judged it — abnormal / no test)."""
    outcome = result.test_outcome
    status = STATUS.get(getattr(outcome, "value", outcome))
    if status is None:
        return None
    replaced = replacement_from_diff(result.diff)
    return {
        "file": str(mutation.module_path).replace("\\", "/"),
        "line": mutation.start_pos[0],
        "status": status,
        "mutator": mutation.operator_name,
        **({} if replaced is None else {"replacement": replaced}),
    }
