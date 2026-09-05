import tomllib
from pathlib import Path


def derive_build_command(config: str, language: str) -> str:
    """The `[<language>].build_command` declaration read from the in-effect config file, or `''`
    when that file is absent or unparseable, `language` is empty, or the table declares no
    string `build_command`."""
    if not language:
        return ""
    path = Path(config)
    if not path.is_file():
        return ""
    try:
        data = tomllib.loads(path.read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return ""
    value = data.get(language, {}).get("build_command", "")
    return value if isinstance(value, str) else ""
