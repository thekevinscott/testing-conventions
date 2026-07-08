"""Colocated unit tests for the red-path-hermetic-wired decision (isolation — pure text in/out)."""
from checks.red_path_hermetic_wired.decide import RED_PATH_JOBS, unwired_jobs

WIRED_STEPS = (
    "    needs: [build-cli]\n"
    "    runs-on: ubuntu-latest\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "      - uses: ./.github/actions/download-hermetic-cli\n"
    "      - run: uv run --project internals/checks tc-checks x\n"
)


def workflow(**overrides: str) -> str:
    """A jobs: mapping with every red-path job wired, save any overridden with a custom block."""
    jobs = "".join(f"  {job}:\n{overrides.get(job, WIRED_STEPS)}" for job in RED_PATH_JOBS)
    # A trailing non-red-path job, so the last red-path job's block is bounded by a real header.
    return "jobs:\n" + jobs + "  some-uses-job:\n    uses: ./.github/workflows/testing-conventions.yml\n"


def test_all_jobs_wired_reports_nothing():
    assert unwired_jobs(workflow()) == []


def test_a_job_missing_the_download_is_reported():
    no_download = (
        "    needs: [build-cli]\n"
        "    steps:\n"
        "      - uses: actions/checkout@v6\n"
        "      - run: uv run --project internals/checks tc-checks x\n"
    )
    assert unwired_jobs(workflow(**{"packaging-red": no_download})) == ["packaging-red"]


def test_a_job_missing_the_build_cli_edge_is_reported():
    no_needs = (
        "    steps:\n"
        "      - uses: actions/checkout@v6\n"
        "      - uses: ./.github/actions/download-hermetic-cli\n"
    )
    assert unwired_jobs(workflow(**{"isolation-red": no_needs})) == ["isolation-red"]


def test_a_job_absent_from_the_file_is_reported():
    text = "jobs:\n" + "".join(
        f"  {job}:\n{WIRED_STEPS}" for job in RED_PATH_JOBS if job != "colocated-rust-red"
    )
    assert unwired_jobs(text) == ["colocated-rust-red"]


def test_a_neighbours_wiring_does_not_satisfy_a_bare_job():
    # `mutation-gate` is bare; the *following* job carries both the edge and the download. Bounding
    # each job to its own block keeps the neighbour's wiring from covering the bare one.
    bare = "    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n"
    assert "mutation-gate" in unwired_jobs(workflow(**{"mutation-gate": bare}))


def test_reports_preserve_red_path_job_order():
    bare = "    steps:\n      - run: echo hi\n"
    result = unwired_jobs(workflow(**{"colocated-rust-red": bare, "below-floor": bare}))
    assert result == ["below-floor", "colocated-rust-red"]


def test_red_path_jobs_names_the_nine_direct_drive_jobs():
    assert set(RED_PATH_JOBS) == {
        "below-floor",
        "mutation-gate",
        "python-mutation-clean",
        "isolation-red",
        "packaging-red",
        "coverage-rust-red",
        "integration-lint-new-arms-trip",
        "packaging-package-root-red",
        "colocated-rust-red",
    }
    assert len(RED_PATH_JOBS) == 9
