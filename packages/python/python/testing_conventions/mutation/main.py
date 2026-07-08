"""Executable entry for the Python mutation arm: the rust binary spawns
``python -m testing_conventions.mutation.main`` and this runs [`mutation_cli`] over the
process arguments. Kept thin so the orchestration stays a pure, importable function.
"""
import sys

from testing_conventions.mutation.cli import mutation_cli

mutation_cli(sys.argv[1:])
