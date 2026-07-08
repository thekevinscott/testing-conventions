# Red barrel test (#382 guard): a barrel test that reaches AROUND the barrel into a
# sibling module directly (`from .core import Thing`) is importing a collaborator,
# not the SUT's own surface — so it is still flagged, exactly as before. The
# exemption is only for the bare `from . import …` that resolves to the SUT file.
from .core import Thing


def describe_barrel():
    def it_reaches_into_core():
        assert Thing is not None
