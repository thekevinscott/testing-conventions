import tomllib
from pathlib import Path


def derive_e2e_extra_scope(config: str) -> str:
    """The `[e2e].extra_scope` roots from the in-effect config, as repeated `--extra-scope` flags."""
    return _e2e_scope_flags(config, "extra_scope", "--extra-scope")


def derive_e2e_exclude(config: str) -> str:
    """The `[e2e].exclude` subtrees from the in-effect config, as repeated `--exclude` flags."""
    return _e2e_scope_flags(config, "exclude", "--exclude")


def _e2e_scope_flags(config: str, key: str, flag: str) -> str:
    """The `[e2e].<key>` list rendered as `<flag> <dir>` pairs, or `''` when the file is
    absent/unparseable or the key is missing or not a list of directory strings. The run step
    word-splits the result, so a path with a space would not survive."""
    path = Path(config)
    if not path.is_file():
        return ""
    try:
        data = tomllib.loads(path.read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return ""
    value = data.get("e2e", {}).get(key, [])
    if not isinstance(value, list):
        return ""
    return " ".join(f"{flag} {directory}" for directory in value if isinstance(directory, str) and directory)
