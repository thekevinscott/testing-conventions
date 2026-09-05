def build_command_language(primary: str, present: list[str]) -> str:
    """The language whose `build_command` table applies: `primary` when there is one, else the
    sole present language when that is unambiguous, else `''` — never a guess."""
    if primary:
        return primary
    try:
        (sole,) = present
    except ValueError:
        return ""
    return sole
