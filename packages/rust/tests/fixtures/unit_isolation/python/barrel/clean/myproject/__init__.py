# A re-export barrel: its public surface is what the colocated `__init___test.py`
# verifies.
from .core import Thing

__all__ = ["Thing", "__version__"]
__version__ = "0.0.0"
