"""Colocated unit test for the job-block iterator (isolation — only the UUT imported).

`iter_job_blocks` is driven over YAML whose job names aren't known ahead of time, plus a scalar
top-level key (`permissions:` / `contents: read`) it must not mistake for a job header.
"""
from checks.utils.job_block import iter_job_blocks

TEXT = """\
jobs:
  build:
    run: before-marker
  target:
    run: inside-marker
  next:
    run: after-marker
"""

TEXT_WITH_SCALAR_KEY = """\
permissions:
  contents: read

jobs:
  first:
    run: first-marker
  second:
    run: second-marker
"""


def test_iter_job_blocks_yields_every_job_bounded_to_its_own_region():
    blocks = dict(iter_job_blocks(TEXT))
    assert list(blocks) == ["build", "target", "next"]
    # The *first* job's own block, not just later ones: a wrong index arithmetic (e.g. reading
    # the current header's own start instead of the next one's) would make this block empty
    # rather than merely shifted, since `build` is index 0.
    assert "build:" in blocks["build"]
    assert "before-marker" in blocks["build"]
    assert "inside-marker" not in blocks["build"]
    assert "inside-marker" in blocks["target"]
    assert "before-marker" not in blocks["target"]
    assert "after-marker" not in blocks["target"]
    # The last job's block has no following header to stop at — it must still run to EOF.
    assert "after-marker" in blocks["next"]


def test_iter_job_blocks_does_not_mistake_a_scalar_top_level_key_for_a_job():
    # `  contents: read` is two-space indented like a job header, but carries a value on the
    # same line — a job header is a bare `<name>:` with nothing after it.
    blocks = dict(iter_job_blocks(TEXT_WITH_SCALAR_KEY))
    assert list(blocks) == ["first", "second"]
    assert "contents" not in blocks
