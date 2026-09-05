from output_delimiter import output_delimiter


def render_github_output(outputs: dict[str, str]) -> str:
    """Render `outputs` as the text appended to `GITHUB_OUTPUT`: `name=value` lines, with the
    runner's heredoc form for a value carrying a newline — a raw `name=value` line there would
    end the file-command line early and parse the remaining lines as bogus outputs."""
    lines: list[str] = []
    for name, value in outputs.items():
        if "\n" in value:
            delim = output_delimiter(value)
            lines.append(f"{name}<<{delim}")
            lines.append(value)
            lines.append(delim)
        else:
            lines.append(f"{name}={value}")
    return "".join(f"{line}\n" for line in lines)
