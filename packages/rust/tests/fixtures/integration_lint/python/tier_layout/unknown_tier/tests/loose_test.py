# A test file directly under `tests/` — neither `tests/integration/` nor
# `tests/e2e/`, so it sits outside every standard tier.
def describe_loose():
    def it_passes():
        assert True
