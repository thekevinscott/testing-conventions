#!/usr/bin/env python3
"""Assert the `coverage-rust-red` self-test job's published-CLI exit codes (epic #302, #309).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `coverage-rust-red` job in
`.github/workflows/testing-conventions-selftest.yml`, which
drives the published `unit coverage --language rust` command over a sub-floor crate and
asserts the non-zero exit that fails a consumer's build, then asserts each exit code.

That decision (run a command, then pass/fail on its exit code) lived as an inline `run: |` bash
block: untested, and exposed to the GitHub-Actions `${{ }}` templating trap (a `run:` body is
templated before the shell sees it). Lifting it here earns it colocated, integration, and e2e tests.

`subprocess` is the one external dependency; it lives behind [`run_command`] so an integration test
can mock the boundary and drive the real `main` orchestration, while an e2e test runs the whole
path against a benign command. The exact npx invocations the job asserts on are hardcoded in
[`CHECKS`]; the workflow step runs this with no arguments. A single command may be passed as
trailing arguments instead — the seam the e2e test drives with a benign `true`/`false`.
"""
from __future__ import annotations

import subprocess
import sys

# The published-CLI invocations this job asserts on. Each entry is
# `(command, expect_fail, label)`: `expect_fail` is True for a red-path check (the command must
# exit non-zero) and False for a clean-path check (it must exit zero).
CHECKS = [
    (
        ["npx", "-y", "testing-conventions", "unit", "coverage", "--language", "rust", "--config", ".github/selftest/coverage-rust/below/testing-conventions.toml", ".github/selftest/coverage-rust/below"],
        True,
        "below-floor crate trips the rust coverage gate",
    ),
]


def run_command(command):
    """Run `command`, letting its output stream to this process's stdio, and return the result.

    The one external boundary: an integration test patches this to fake the exit code, and an e2e
    test runs it for real against a benign command.
    """
    return subprocess.run(command)


def expect_failure(returncode):
    """A red-path assertion: `None` when the command failed as required (a truthy, non-zero exit
    code), else the reason it did not. Truthiness — not `== 0` — so no comparison operator can
    mutate into an equivalent (`<= 0` never differs for a real exit code; `is 0` never differs for
    a small cached int)."""
    if not returncode:
        return "the command exited 0, but a non-zero (failing) exit was required"
    return None


def expect_success(returncode):
    """A clean-path assertion: `None` when the command passed as required (a falsy, zero exit code),
    else the reason it did not. Truthiness for the same reason as [`expect_failure`]."""
    if returncode:
        return f"the command exited {returncode}, but a zero (passing) exit was required"
    return None


def evaluate(expect_fail, returncode):
    """Dispatch to the red- or clean-path assertion; returns an error message or `None`."""
    if expect_fail:
        return expect_failure(returncode)
    return expect_success(returncode)


def parse(argv):
    """The checks to run: the hardcoded [`CHECKS`], or — when trailing arguments are given — the
    single command in `argv[1:]` treated as a red-path check. The latter is the seam an e2e test
    drives with a benign command; the slice's truthiness carries no comparison operator to mutate
    (`len(argv) > 1` would mutate to an equivalent `!= 1`, since `argv` always holds the program
    name)."""
    command = argv[1:]
    if command:
        return [(command, True, "cli")]
    return CHECKS


def main(argv):
    """Run every check, printing an `::error::` line for each that did not meet its expectation.
    Returns 1 if any check failed its expectation, else 0."""
    failed = 0
    for command, expect_fail, label in parse(argv):
        reason = evaluate(expect_fail, run_command(command).returncode)
        if reason is None:
            print(f"[{label}] ok")
        else:
            print(f"::error::[{label}] {reason}")
            failed = 1
    return failed


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
