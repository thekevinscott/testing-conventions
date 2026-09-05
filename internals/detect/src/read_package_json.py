import json
from pathlib import Path


def read_package_json(root: Path) -> dict:
    """The parsed `package.json` at `root`, or `{}` if absent or unparseable."""
    manifest = root / "package.json"
    if not manifest.is_file():
        return {}
    try:
        return json.loads(manifest.read_text())
    except (OSError, json.JSONDecodeError):
        return {}
