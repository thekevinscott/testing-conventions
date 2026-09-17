import pytest


# The package-level integration tier. The unit mutation gate judges mutants by the scan
# path's colocated suite alone; if it ever runs this tier, the clean (baseline) run fails loudly.
def test_is_never_run_by_the_unit_mutation_gate():
    pytest.fail("the unit mutation gate ran a package-level suite tier")
