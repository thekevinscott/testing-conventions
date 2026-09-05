import render_github_output


def _parse_github_output(text):
    """Parse a GITHUB_OUTPUT file the way the Actions runner does: `name=value` lines
    plus the heredoc `name<<DELIM` / body / `DELIM` form for multi-line values."""
    result = {}
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if "<<" in line and "=" not in line.split("<<", 1)[0]:
            name, delim = line.split("<<", 1)
            i += 1
            body = []
            while i < len(lines) and lines[i] != delim:
                body.append(lines[i])
                i += 1
            result[name] = "\n".join(body)
            i += 1  # skip the closing delimiter
        elif "=" in line:
            name, value = line.split("=", 1)
            result[name] = value
            i += 1
        else:
            i += 1  # blank/trailing line
    return result


def test_render_github_output_writes_single_line_values_as_name_equals_value():
    rendered = render_github_output.render_github_output(
        {"languages": '["python"]', "package_root": "."}
    )
    assert _parse_github_output(rendered) == {"languages": '["python"]', "package_root": "."}


def test_render_github_output_round_trips_a_multiline_value():
    outputs = {
        "build_command": "cp a.tmpl a.py\ncp b.tmpl b.py",
        "languages": '["python"]',
    }
    rendered = render_github_output.render_github_output(outputs)
    assert _parse_github_output(rendered) == outputs
