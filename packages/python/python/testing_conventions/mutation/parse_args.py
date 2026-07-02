"""Parse the mutation adapter's CLI arguments.

The rust binary spawns ``python -m testing_conventions.mutation.main`` and passes these:
``--out <path>`` (where to write the normalized-results JSON — the rule passes a temp file)
and ``--module <path>`` (repeatable; the source files to mutate, for a diff-scoped run).
Without any ``--module`` the whole project is mutated.
"""
from __future__ import annotations

import argparse
from dataclasses import dataclass, field


@dataclass
class Args:
    """The adapter's parsed arguments."""

    out: str
    modules: list[str] = field(default_factory=list)


def parse_args(argv):
    """Parse ``argv`` (a list of strings) into [`Args`]."""
    parser = argparse.ArgumentParser(prog="testing-conventions-mutation")
    parser.add_argument("--out", required=True)
    parser.add_argument("--module", action="append", default=[], dest="modules")
    ns = parser.parse_args(argv)
    return Args(out=ns.out, modules=ns.modules)
