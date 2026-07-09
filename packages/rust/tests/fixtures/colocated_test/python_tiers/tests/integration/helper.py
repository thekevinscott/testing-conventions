# A suite helper with real logic and no colocated twin: the suite tiers belong
# to the integration checks, so the colocated-unit rule holds no claim here.
def load_fixture(name: str) -> str:
    return f"fixtures/{name}.json"
