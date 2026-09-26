"""Colocated unit tests for the instructions-file read (isolation — injected opener)."""
from checks.agents_md_size.fs_ops import read_text


class _File:
    """The slice of `pathlib.Path` this wrapper reads."""

    def __init__(self, text):
        self.text = text
        self.seen = []

    def read_text(self, **kwargs):
        self.seen.append(kwargs)
        if isinstance(self.text, OSError):
            raise self.text
        return self.text


def _opener_returning(text):
    """A fake `pathlib.Path` that records the parts it was built from and yields fixed text."""
    file = _File(text)

    def opener(*parts):
        file.parts = parts
        return file

    opener.file = file
    return opener


def test_read_text_returns_the_files_contents():
    opener = _opener_returning("prose\n")
    assert read_text("root", "AGENTS.md", opener=opener) == "prose\n"
    assert opener.file.parts == ("root", "AGENTS.md")


def test_read_text_decodes_as_utf_8():
    opener = _opener_returning("prose\n")
    read_text("root", "AGENTS.md", opener=opener)
    assert opener.file.seen == [{"encoding": "utf-8"}]


def test_an_empty_file_reads_as_empty_text():
    assert read_text("root", "AGENTS.md", opener=_opener_returning("")) == ""


def test_a_missing_file_reads_as_none():
    assert read_text("root", "gone.md", opener=_opener_returning(FileNotFoundError())) is None


def test_a_directory_in_the_files_place_reads_as_none():
    assert read_text("root", "pkg", opener=_opener_returning(IsADirectoryError())) is None
