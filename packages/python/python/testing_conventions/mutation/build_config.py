"""Parse the rendered TOML into the object cosmic-ray's ``init`` / ``execute`` consume.

``cosmic_ray`` is imported lazily so this module imports without the engine installed.
"""
from __future__ import annotations

from .config import render_config


def build_config(modules, timeout):
    """The parsed cosmic-ray ``ConfigDict`` for a run over ``modules`` with ``timeout``."""
    from cosmic_ray.config import deserialize_config

    return deserialize_config(render_config(modules, timeout))
