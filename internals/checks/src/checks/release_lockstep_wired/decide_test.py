"""Colocated unit tests for the release-lockstep decision (isolation — pure data in/out)."""
from checks.release_lockstep_wired.decide import PUBLISHED_KINDS, lockstep_error

CRATE = {"name": "tc-rust", "kind": "crates", "path": "packages/rust", "globs": ["packages/rust/**"]}


def published(kind, name, globs=("packages/rust/**",), depends_on=("tc-rust",)):
    return {"name": name, "kind": kind, "globs": list(globs), "depends_on": list(depends_on)}


def config(*packages):
    return [CRATE, *packages]


def test_both_halves_rebuilding_from_the_crate_pass():
    assert lockstep_error(config(published("pypi", "tc-py"), published("npm", "tc-npm"))) is None


def test_a_half_omitting_the_crate_glob_fails_and_names_both():
    error = lockstep_error(config(published("pypi", "tc-py", globs=["packages/python/**"])))
    assert "tc-py" in error
    assert "packages/rust/**" in error
    assert "omits" in error


def test_the_crate_glob_is_derived_from_the_crate_path_not_assumed():
    crate = {**CRATE, "path": "crates/cli"}
    assert lockstep_error([crate, published("npm", "tc-npm", globs=["crates/cli/**"])]) is None
    assert lockstep_error([crate, published("npm", "tc-npm")]) is not None


def test_a_half_not_depending_on_the_crate_fails_and_names_both():
    error = lockstep_error(config(published("npm", "tc-npm", depends_on=[])))
    assert "tc-npm" in error
    assert "tc-rust" in error
    assert "does not depend on" in error


def test_every_published_half_is_checked_not_just_the_first():
    error = lockstep_error(config(published("pypi", "tc-py"), published("npm", "tc-npm", globs=[])))
    assert error is not None
    assert "tc-npm" in error


def test_a_package_of_another_kind_is_neither_the_crate_nor_a_published_half():
    unreleased = {"name": "tc-action", "kind": "action", "globs": [".github/actions/**"]}
    assert lockstep_error([*config(published("npm", "tc-npm")), unreleased]) is None


def test_a_config_publishing_nothing_fails():
    assert "npm or pypi" in lockstep_error(config())


def test_a_config_with_no_crate_fails():
    assert "0 packages have kind `crates`" in lockstep_error([published("npm", "tc-npm")])


def test_a_config_with_two_crates_fails():
    packages = [CRATE, {**CRATE, "name": "other"}, published("npm", "tc-npm")]
    assert "2 packages have kind `crates`" in lockstep_error(packages)


def test_published_kinds_are_the_two_registries_a_consumer_installs_from():
    assert PUBLISHED_KINDS == ("npm", "pypi")
