"""Parse the adapter's ``--lines`` specs into per-module line ranges."""
from __future__ import annotations


def parse_line_ranges(specs):
    """Map each ``<file>:<start>-<end>`` spec in ``specs`` to its module, as a dict of file to
    inclusive ``(start, end)`` ranges. The last ``:`` splits, so a path may carry its own."""
    ranges = {}
    for spec in specs:
        file, span = spec.rsplit(":", 1)
        start, end = span.split("-")
        ranges.setdefault(file, []).append((int(start), int(end)))
    return ranges
