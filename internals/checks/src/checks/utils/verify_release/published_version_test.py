"""Colocated unit tests for the published-version pick (isolation — pure tags in/out)."""
from checks.utils.verify_release.published_version import published_version


def test_published_version_picks_the_numeric_max_not_the_lexical_one():
    tags = [
        "testing-conventions-npm-v0.0.9",
        "testing-conventions-npm-v0.0.67",
        "testing-conventions-npm-v0.0.8",
    ]
    assert published_version(tags) == "0.0.67"


def test_published_version_ignores_non_npm_tags():
    assert published_version(["testing-conventions-rust-v0.0.99", "testing-conventions-npm-v0.0.2"]) == "0.0.2"


def test_published_version_raises_when_no_npm_tag_is_present():
    try:
        published_version(["testing-conventions-rust-v0.0.1", "v0"])
    except ValueError as error:
        assert "refusing to promote" in str(error)
    else:
        raise AssertionError("no npm tag must raise")
