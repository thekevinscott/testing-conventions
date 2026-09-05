import hashlib


def free_of(base: str, lines: list[str]) -> str:
    """`base`, suffixed with a counter until it matches no entry of `lines`."""
    delim = base
    counter = 0
    while delim in lines:
        counter += 1
        delim = f"{base}_{counter}"
    return delim


def output_delimiter(value: str) -> str:
    """A deterministic heredoc delimiter that appears on no line of `value`."""
    return free_of("ghadelimiter_" + hashlib.sha256(value.encode()).hexdigest(), value.split("\n"))
