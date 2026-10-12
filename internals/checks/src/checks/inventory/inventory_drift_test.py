from checks.inventory.inventory_drift import inventory_drift

AGREEING = [
    {
        "gate": "unit-coverage",
        "command": "unit coverage",
        "jobs": ["Unit-test coverage", "Unit-test coverage — changed lines"],
    },
    {"jobs": ["Detect requested languages with sources"]},
    {"command": "install"},
]
GATES = ["unit-coverage"]
JOBS = [
    "Unit-test coverage",
    "Unit-test coverage — changed lines",
    "Detect requested languages with sources",
]
COMMANDS = ["unit coverage", "install"]


def test_an_inventory_matching_every_surface_has_no_drift() -> None:
    assert inventory_drift(AGREEING, JOBS, COMMANDS, GATES) == []


def test_a_gate_the_workflow_gained_is_reported() -> None:
    assert inventory_drift(AGREEING, JOBS, COMMANDS, [*GATES, "packaging"]) == [
        '"packaging" is in the gates input and not in the inventory',
    ]


def test_a_gate_the_inventory_claims_and_the_workflow_dropped_is_reported() -> None:
    assert inventory_drift(AGREEING, JOBS, COMMANDS, []) == [
        'the gates input has no "unit-coverage", and the inventory claims it',
    ]


def test_a_job_the_workflow_gained_is_reported() -> None:
    assert inventory_drift(AGREEING, [*JOBS, "Benchmark"], COMMANDS, GATES) == [
        '"Benchmark" is in the workflow and not in the inventory',
    ]


def test_a_command_the_cli_gained_is_reported() -> None:
    assert inventory_drift(AGREEING, JOBS, [*COMMANDS, "e2e slug"], GATES) == [
        '"e2e slug" is in the CLI and not in the inventory',
    ]


def test_a_job_the_workflow_dropped_is_reported() -> None:
    assert inventory_drift(AGREEING, JOBS[:-1], COMMANDS, GATES) == [
        'the workflow has no "Detect requested languages with sources", and the inventory claims it',
    ]


def test_a_command_the_cli_dropped_is_reported() -> None:
    assert inventory_drift(AGREEING, JOBS, ["install"], GATES) == [
        'the CLI has no "unit coverage", and the inventory claims it',
    ]


def test_an_entry_carrying_a_job_and_no_command_claims_neither_command_nor_gate() -> None:
    entries = [{"jobs": ["Detect requested languages with sources"]}]
    assert inventory_drift(entries, ["Detect requested languages with sources"], [], []) == []


def test_an_entry_carrying_a_command_and_no_job_claims_neither_job_nor_gate() -> None:
    assert inventory_drift([{"command": "install"}], [], ["install"], []) == []


def test_one_check_may_claim_several_jobs() -> None:
    entries = [{"gate": "unit-coverage", "command": "unit coverage", "jobs": ["whole", "changed"]}]
    assert inventory_drift(entries, ["whole", "changed"], ["unit coverage"], ["unit-coverage"]) == []


def test_every_surface_is_reported_and_the_gates_input_comes_first() -> None:
    drift = inventory_drift(AGREEING, [*JOBS, "Benchmark"], [*COMMANDS, "e2e slug"], [*GATES, "ghost"])
    assert [line.split('"')[1] for line in drift] == ["ghost", "Benchmark", "e2e slug"]
    assert "the gates input" in drift[0]
    assert "the workflow" in drift[1]
    assert "the CLI" in drift[2]


def test_an_empty_inventory_reports_every_name_the_surfaces_have() -> None:
    assert inventory_drift([], ["Build"], ["packaging"], ["packaging"]) == [
        '"packaging" is in the gates input and not in the inventory',
        '"Build" is in the workflow and not in the inventory',
        '"packaging" is in the CLI and not in the inventory',
    ]


def test_the_entries_are_read_once_so_an_iterator_is_accepted() -> None:
    assert inventory_drift(iter(AGREEING), JOBS, COMMANDS, GATES) == []
