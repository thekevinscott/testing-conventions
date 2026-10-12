from checks.inventory.claimed_names import claimed_names


def test_reads_a_scalar_field_as_one_name() -> None:
    assert claimed_names([{"command": "unit coverage"}], "command") == ["unit coverage"]


def test_flattens_a_list_valued_field_into_its_members() -> None:
    entries = [{"jobs": ["Static checks", "Unit-test coverage"]}]
    assert claimed_names(entries, "jobs") == ["Static checks", "Unit-test coverage"]


def test_reads_every_entry_not_only_the_first() -> None:
    entries = [{"command": "packaging"}, {"command": "changelog"}]
    assert claimed_names(entries, "command") == ["packaging", "changelog"]


def test_skips_an_entry_that_carries_no_such_field() -> None:
    entries = [{"command": "unit lint"}, {"jobs": ["Detect requested languages with sources"]}]
    assert claimed_names(entries, "command") == ["unit lint"]


def test_keeps_a_repeated_name_once_per_entry_that_claims_it() -> None:
    entries = [{"jobs": ["Static checks"]}, {"jobs": ["Static checks"]}]
    assert claimed_names(entries, "jobs") == ["Static checks", "Static checks"]


def test_an_empty_list_claims_nothing() -> None:
    assert claimed_names([{"jobs": []}], "jobs") == []


def test_no_entries_claim_nothing() -> None:
    assert claimed_names([], "command") == []
