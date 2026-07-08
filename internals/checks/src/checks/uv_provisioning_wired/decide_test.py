"""Colocated unit tests for the uv-provisioning-wired decision (isolation — pure text in/out)."""
from checks.uv_provisioning_wired.decide import SUITE_JOBS, decide, python_steps

PYTHON_ARM = (
    "      - if: matrix.language == 'python'\n"
    "        uses: astral-sh/setup-uv@v7\n"
    "        with:\n"
    "          python-version: '3.12'\n"
    "      - if: matrix.language == 'python'\n"
    "        name: Provision the Python suite environment (uv)\n"
    "        run: |\n"
    "          uv sync\n"
)


def job(name: str, python_arm: str = PYTHON_ARM, extra: str = "") -> str:
    return f"  {name}:\n    steps:\n      - uses: actions/checkout@v6\n{extra}{python_arm}"


def workflow(unit_coverage: str = PYTHON_ARM, coverage_changed: str = PYTHON_ARM, mutation: str = PYTHON_ARM) -> str:
    return (
        "jobs:\n"
        + job("unit-coverage", unit_coverage)
        + job("coverage-changed", coverage_changed)
        + job("mutation", mutation)
        + "  integration-lint:\n    steps:\n      - run: echo lint\n"
    )


def test_identical_uv_arms_pass():
    assert decide(workflow())


def test_a_pip_install_fails_even_with_identical_arms():
    assert not decide(workflow() + "      - run: python -m pip install --quiet coverage pytest\n")


def test_a_setup_python_fails_even_with_identical_arms():
    assert not decide(workflow() + "      - uses: actions/setup-python@v6\n")


def test_a_lexicographically_greater_drift_fails():
    # "uv zzzz" sorts after "uv sync" — an arm that is *greater* than the canonical one, so a
    # `==` mutated to `>=` would wrongly accept it. Drift each rest position.
    greater = PYTHON_ARM.replace("uv sync", "uv zzzz")
    assert not decide(workflow(mutation=greater))
    assert not decide(workflow(coverage_changed=greater))


def test_a_lexicographically_lesser_drift_fails():
    # "uv aaaa" sorts before "uv sync" — an arm that is *less* than the canonical one, so a
    # `==` mutated to `<=` would wrongly accept it.
    lesser = PYTHON_ARM.replace("uv sync", "uv aaaa")
    assert not decide(workflow(mutation=lesser))
    assert not decide(workflow(coverage_changed=lesser))


def test_a_drifted_unit_coverage_arm_fails():
    # Drift the *first* arm: the two others still match each other but not it, so a comparison
    # anchored on the wrong arm would wrongly accept.
    assert not decide(workflow(unit_coverage=PYTHON_ARM.replace("uv sync", "uv zzzz")))


def test_jobs_with_no_python_arm_fail():
    assert not decide(workflow("", "", ""))


def test_a_comment_only_difference_still_passes():
    commented = "      # the same steps, annotated for this job\n" + PYTHON_ARM
    assert decide(workflow(unit_coverage=commented))


def test_python_steps_picks_only_python_gated_step_chunks():
    block = (
        "  unit-coverage:\n"
        "    steps:\n"
        "      - uses: actions/checkout@v6\n"
        "      - if: matrix.language == 'python'\n"
        "        run: uv sync\n"
        "      - if: matrix.language == 'typescript'\n"
        "        run: npm ci\n"
    )
    assert python_steps(block) == "      - if: matrix.language == 'python'\n        run: uv sync"


def test_python_steps_drops_comment_and_blank_lines_inside_a_chunk():
    block = (
        "      - if: matrix.language == 'python'\n"
        "        # provision with uv\n"
        "\n"
        "        run: uv sync\n"
    )
    assert python_steps(block) == "      - if: matrix.language == 'python'\n        run: uv sync"


def test_python_steps_is_empty_when_no_chunk_is_python_gated():
    assert python_steps("      - uses: actions/checkout@v6\n") == ""


def test_suite_jobs_name_the_three_suite_executing_jobs_with_their_successors():
    assert SUITE_JOBS == (
        ("unit-coverage", "coverage-changed"),
        ("coverage-changed", "mutation"),
        ("mutation", "integration-lint"),
    )
