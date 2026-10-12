from checks.inventory.surface_drift import surface_drift


def test_agreement_is_no_drift() -> None:
    assert surface_drift("the CLI", ["packaging"], ["packaging"]) == []


def test_a_name_the_inventory_claims_and_the_surface_lacks_is_reported() -> None:
    assert surface_drift("the CLI", ["packaging", "ghost"], ["packaging"]) == [
        'the CLI has no "ghost", and the inventory claims it',
    ]


def test_a_name_the_surface_has_and_the_inventory_lacks_is_reported() -> None:
    assert surface_drift("the workflow", ["Static checks"], ["Static checks", "Deploy"]) == [
        '"Deploy" is in the workflow and not in the inventory',
    ]


def test_both_directions_are_reported_together() -> None:
    assert surface_drift("the CLI", ["ghost"], ["real"]) == [
        'the CLI has no "ghost", and the inventory claims it',
        '"real" is in the CLI and not in the inventory',
    ]


def test_the_surface_is_named_in_every_message() -> None:
    assert all("the workflow" in line for line in surface_drift("the workflow", ["a"], ["b"]))


def test_names_are_reported_in_sorted_order() -> None:
    drift = surface_drift("the CLI", ["z", "a"], [])
    assert drift == [
        'the CLI has no "a", and the inventory claims it',
        'the CLI has no "z", and the inventory claims it',
    ]


def test_a_duplicate_claim_is_reported_once() -> None:
    assert surface_drift("the CLI", ["ghost", "ghost"], []) == [
        'the CLI has no "ghost", and the inventory claims it',
    ]


def test_two_empty_surfaces_agree() -> None:
    assert surface_drift("the CLI", [], []) == []
