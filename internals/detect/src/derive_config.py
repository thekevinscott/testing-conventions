from pathlib import Path

CONFIG_DEFAULT = "testing-conventions.toml"


def derive_config(package_root_rel: Path, config_input: str) -> str:
    """The config file `--config` should receive: `config_input` verbatim when the caller named
    anything other than the default, else the package root's own `testing-conventions.toml`
    when that file exists, else the default itself."""
    if config_input != CONFIG_DEFAULT:
        return config_input
    candidate = package_root_rel / CONFIG_DEFAULT
    return str(candidate) if candidate.is_file() else CONFIG_DEFAULT
