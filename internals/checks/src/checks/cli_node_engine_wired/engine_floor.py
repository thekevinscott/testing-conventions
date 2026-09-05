"""Read the node major an `engines.node` requirement demands."""
from __future__ import annotations

import re

from checks.utils.check_failed import CheckFailed


def engine_floor(requirement: str) -> int:
    """The major node version an `engines.node` `requirement` demands, e.g. 24 from `>=24`."""
    match = re.fullmatch(r">=\s*(\d+)(?:\.\d+)*", requirement.strip())
    if match is None:
        raise CheckFailed(f"`engines.node` is `{requirement}`, not the `>=<major>` floor this check reads")
    return int(match.group(1))
