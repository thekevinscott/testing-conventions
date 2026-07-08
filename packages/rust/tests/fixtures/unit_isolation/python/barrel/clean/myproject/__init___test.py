# Clean barrel test (#382): a re-export barrel can only be tested by importing its
# public surface, so `from . import …` names the package's own `__init__.py` — the
# unit under test — and none of these are collaborators. `__all__` / `__version__`
# are defined in the SUT itself. Parity with TS's `index.test.ts` importing
# `./index.js`.
from . import Thing, __all__, __version__


def describe_barrel():
    def it_reexports_its_surface():
        assert Thing is not None
        assert __version__ == "0.0.0"

    def it_declares_all():
        assert set(__all__) == {"Thing", "__version__"}
