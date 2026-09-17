import json
import os

from dirsql import DirSQL, Table


def _read(path):
    """Return a file's text, or None if it doesn't exist."""
    try:
        with open(path) as f:
            return f.read()
    except FileNotFoundError:
        return None


def extract_source(path):
    """One row per source folder, keyed off its metadata.json.

    `path` is the matched metadata.json; its siblings are the capture artifacts.
    Carries the three artifact files' contents plus the parsed relevance fields.
    relevance.md is absent for sources judged not relevant -> NULL.
    """
    folder = os.path.dirname(path)
    meta_raw = _read(path)
    meta = json.loads(meta_raw)
    return [
        {
            "summary": _read(os.path.join(folder, "summary.md")),
            "relevance": _read(os.path.join(folder, "relevance.md")),
            "metadata": meta_raw,
            "relevant": meta.get("relevant"),
            "reason": meta.get("reason"),
        }
    ]


# Python must export an `app` variable
app = DirSQL(
    tables=[
        Table(
            ddl=(
                "CREATE TABLE sources ("
                "_dir TEXT, "       # source folder (the slug), relative to scan root
                "summary TEXT, "    # contents of summary.md
                "relevance TEXT, "  # contents of relevance.md (NULL if not relevant)
                "metadata TEXT, "   # raw contents of metadata.json
                "relevant BOOL, "   # parsed from metadata.json
                "reason TEXT"       # parsed from metadata.json
                ")"
            ),
            glob="**/metadata.json",
            extract=extract_source,
        ),
    ],
)
