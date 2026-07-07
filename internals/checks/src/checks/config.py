"""Shared paths the checks target (#321) — one source of truth for the workflow files a check
reads, so the literal isn't copied into every check module.
"""
REUSABLE_WORKFLOW = ".github/workflows/testing-conventions.yml"
DOGFOOD_HELPERS_WORKFLOW = ".github/workflows/dogfood-github-helpers.yml"
