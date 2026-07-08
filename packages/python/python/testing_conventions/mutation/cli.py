"""The mutation adapter's orchestrator, the Python counterpart of the TypeScript
``mutationCLI``: parse args, drive cosmic-ray in-process, and emit the normalized-results
JSON the rust core gates on. A pure, importable function; ``main.py`` is the executable that
runs it.
"""
from __future__ import annotations

import json
import sys

from testing_conventions.mutation.baseline import check_baseline
from testing_conventions.mutation.config import build_config
from testing_conventions.mutation.normalize import normalize
from testing_conventions.mutation.parse_args import parse_args
from testing_conventions.mutation.session import run_session


def mutation_cli(argv):
    """Run the adapter over ``argv``: build the cosmic-ray config, check the baseline, run the
    session, and write the normalized results to ``--out``. Any failure is printed to stderr
    and turned into a non-zero exit code."""
    try:
        args = parse_args(argv)
        config = build_config(args.modules)
        check_baseline(config)
        results = [
            mutant
            for mutation, result in run_session(config)
            if (mutant := normalize(mutation, result)) is not None
        ]
        with open(args.out, "w", encoding="utf-8") as handle:
            json.dump(results, handle)
    except Exception as error:  # surface any adapter failure as a clean non-zero exit
        sys.stderr.write(f"{error}\n")
        sys.exit(1)
