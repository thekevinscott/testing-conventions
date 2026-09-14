import sys
from pathlib import Path

# An editable install resolves through a .pth to the real source tree; the equivalent
# here is a sys.path entry pointing at the neighbor package outside the scanned root.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "a"))
